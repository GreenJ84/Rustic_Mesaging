use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};

use super::super::common::database_connection;
use crate::infra::{DbConn, DbError, DieselError};
use crate::member::models::NewMember;
use crate::member::repository::MemberRepository;
use crate::schema::member::dsl as m_dsl;
use crate::utils::Validator;

struct Identity {
    username: String,
    email: String,
}

impl Identity {
    fn unique(prefix: &str) -> Self {
        let suffix = uuid::Uuid::now_v7().simple().to_string();
        Self {
            username: format!("{prefix}_{suffix}"),
            email: format!("{prefix}_{suffix}@example.com"),
        }
    }
}

fn insert_member(
    conn: &mut DbConn,
    identity: &Identity,
    password_hash: &str,
) -> crate::member::models::MemberRow {
    MemberRepository::create(
        conn,
        NewMember::new(
            identity.username.clone(),
            password_hash.to_string(),
            identity.email.clone(),
        ),
    )
    .expect("member fixture should insert")
}

fn rollback_test(mut test: impl FnMut(&mut DbConn) -> Result<(), DbError>) {
    const MAX_ATTEMPTS: usize = 5;

    for attempt in 1..=MAX_ATTEMPTS {
        let mut conn = database_connection();
        let result = conn.transaction::<(), DbError, _>(|conn| {
            test(conn)?;
            Err(DbError::Transaction(DieselError::RollbackTransaction))
        });

        match result {
            Err(DbError::Transaction(DieselError::RollbackTransaction)) => return,
            Err(DbError::RetryableTransaction(_)) if attempt < MAX_ATTEMPTS => {
                std::thread::yield_now();
            }
            Err(error) => panic!("member repository test transaction failed: {error:?}"),
            Ok(()) => unreachable!("test transaction always requests rollback"),
        }
    }
}

fn delete_members(conn: &mut DbConn, member_ids: &[i32]) {
    diesel::delete(m_dsl::member.filter(m_dsl::id.eq_any(member_ids)))
        .execute(conn)
        .expect("member search fixtures should be cleaned up");
}

#[test]
fn create_and_read_preserve_the_full_member_row() {
    rollback_test(|conn| {
        let identity = Identity::unique("repository_read");
        let created = insert_member(conn, &identity, "original-hash");

        assert!(MemberRepository::exists(conn, created.id())?);
        let read = MemberRepository::read(conn, created.id())?.expect("member should exist");
        assert_eq!(read.username(), identity.username);
        assert_eq!(read.email(), identity.email);
        assert_eq!(read.password_hash(), "original-hash");
        assert_eq!(read.avatar(), "default");
        assert!(MemberRepository::read(conn, i32::MAX)?.is_none());
        Ok(())
    });
}

#[test]
fn detailed_and_short_reads_return_the_intended_projections() {
    rollback_test(|conn| {
        let identity = Identity::unique("repository_projection");
        let created = insert_member(conn, &identity, "private-hash");

        let detailed = MemberRepository::read_detailed(conn, created.id())?
            .expect("detailed member should exist");
        assert_eq!(detailed.id, created.id());
        assert_eq!(detailed.username, identity.username);
        assert_eq!(detailed.email, identity.email);
        Ok(())
    });
}

#[test]
fn identity_lookup_detects_username_and_email_independently() {
    rollback_test(|conn| {
        let identity = Identity::unique("repository_identity");
        let created = insert_member(conn, &identity, "hash");

        assert_eq!(
            MemberRepository::identity_exists(conn, &identity.username, "unused@example.com")?,
            Some(identity.username.clone())
        );
        assert_eq!(
            MemberRepository::identity_exists(conn, "unused-name", &identity.email)?,
            Some(identity.username.clone())
        );
        assert_eq!(
            MemberRepository::identity_exists_excluding(
                conn,
                created.id(),
                &identity.username,
                &identity.email,
            )?,
            None
        );
        Ok(())
    });
}

#[test]
fn duplicate_usernames_are_reported_as_database_conflicts() {
    rollback_test(|conn| {
        let identity = Identity::unique("repository_duplicate_username");
        insert_member(conn, &identity, "hash");
        let duplicate = NewMember::new(
            identity.username.clone(),
            "other-hash".into(),
            Identity::unique("different_email").email,
        );

        assert!(matches!(
            MemberRepository::create(conn, duplicate),
            Err(DbError::Conflict(_))
        ));
        Ok(())
    });
}

#[test]
fn duplicate_emails_are_reported_as_database_conflicts() {
    rollback_test(|conn| {
        let identity = Identity::unique("repository_duplicate_email");
        insert_member(conn, &identity, "hash");
        let duplicate = NewMember::new(
            Identity::unique("different_username").username,
            "other-hash".into(),
            identity.email.clone(),
        );

        assert!(matches!(
            MemberRepository::create(conn, duplicate),
            Err(DbError::Conflict(_))
        ));
        Ok(())
    });
}

#[test]
fn password_update_requires_the_current_hash() {
    rollback_test(|conn| {
        let identity = Identity::unique("repository_password");
        let created = insert_member(conn, &identity, "original-hash");

        assert!(!MemberRepository::update_password(
            conn,
            created.id(),
            "wrong-hash",
            "replacement-hash".into(),
        )?);
        assert!(MemberRepository::update_password(
            conn,
            created.id(),
            "original-hash",
            "replacement-hash".into(),
        )?);
        assert!(!MemberRepository::update_password(
            conn,
            created.id(),
            "original-hash",
            "third-hash".into(),
        )?);
        assert_eq!(
            MemberRepository::read(conn, created.id())?
                .expect("member should remain")
                .password_hash(),
            "replacement-hash"
        );
        Ok(())
    });
}

#[test]
fn search_orders_exact_prefix_case_insensitive_and_contains_matches() {
    let mut conn = database_connection();
    let token = format!("rank{}", uuid::Uuid::now_v7().simple());
    let expected = [
        token.clone(),
        format!("{token}suffix"),
        format!("{}suffix", token.to_uppercase()),
        format!("x{token}"),
    ];
    let member_ids = expected
        .iter()
        .map(|username| {
            insert_member(
                &mut conn,
                &Identity {
                    username: username.clone(),
                    email: format!("{}@example.com", uuid::Uuid::now_v7().simple()),
                },
                "hash",
            )
            .id()
        })
        .collect::<Vec<_>>();

    let results = MemberRepository::search_usernames(
        &mut conn,
        Validator::validate_search_term(token).unwrap(),
    )
    .expect("ranked member search should execute");
    delete_members(&mut conn, &member_ids);

    let usernames = results
        .into_iter()
        .map(|member| member.username)
        .collect::<Vec<_>>();
    assert_eq!(usernames, expected);
}

#[test]
fn search_escapes_sql_wildcards() {
    let mut conn = database_connection();
    let identity = Identity::unique("literal%member");
    let wildcard_id = insert_member(&mut conn, &identity, "hash").id();
    let ordinary_id = insert_member(&mut conn, &Identity::unique("ordinary_member"), "hash").id();

    let results = MemberRepository::search_usernames(
        &mut conn,
        Validator::validate_search_term(identity.username.clone()).unwrap(),
    )
    .expect("escaped member search should execute");
    delete_members(&mut conn, &[wildcard_id, ordinary_id]);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, identity.username);
    assert!(results.iter().all(|member| member.username.contains('%')));
}

#[test]
fn search_limits_results_to_twenty_five_members() {
    let mut conn = database_connection();
    let token = format!("limit{}", uuid::Uuid::now_v7().simple());
    let member_ids = (0..30)
        .map(|index| {
            insert_member(
                &mut conn,
                &Identity {
                    username: format!("{token}{index:02}"),
                    email: format!("{}@example.com", uuid::Uuid::now_v7().simple()),
                },
                "hash",
            )
            .id()
        })
        .collect::<Vec<_>>();

    let results = MemberRepository::search_usernames(
        &mut conn,
        Validator::validate_search_term(token.clone()).unwrap(),
    )
    .expect("limited member search should execute");
    delete_members(&mut conn, &member_ids);

    assert_eq!(results.len(), 25);
    assert_eq!(results.first().unwrap().username, format!("{token}00"));
    assert_eq!(results.last().unwrap().username, format!("{token}24"));
}
