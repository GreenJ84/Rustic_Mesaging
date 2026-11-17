//! Rollback-isolated persistence tests for friendships.

use super::models::NewFriend;
use super::repository::FriendRepository;
use crate::infra::DbError;
use crate::schema::member::dsl as member_dsl;
use crate::tests::social::support::{insert_member, rollback_test};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use shared::models::social::friend::{Friend, FriendDetails, FriendShort};

#[test]
fn friend_repository_covers_canonical_crud_and_projections() {
    rollback_test(|conn| {
        let first = insert_member(conn, "friend_repo_first")?;
        let second = insert_member(conn, "friend_repo_second")?;
        let created = FriendRepository::create(conn, NewFriend::new(second.id(), first.id()))?;
        let created = Friend::from(created);
        assert_eq!(created.member_id, first.id().min(second.id()));
        assert_eq!(created.friend_id, first.id().max(second.id()));

        assert!(FriendRepository::exists(conn, first.id(), second.id())?);
        assert!(FriendRepository::exists(conn, second.id(), first.id())?);
        assert!(FriendRepository::read(conn, second.id(), first.id())?.is_some());

        let detailed = FriendRepository::read_detailed(conn, first.id(), second.id())?
            .map(FriendDetails::from)
            .expect("friendship should have a detailed projection");
        assert_eq!(detailed.friend.id, second.id());
        let inverse = FriendRepository::read_detailed(conn, second.id(), first.id())?
            .map(FriendDetails::from)
            .expect("inverse friendship should have a detailed projection");
        assert_eq!(inverse.friend.id, first.id());

        let detailed_many = FriendRepository::read_many_detailed(conn, first.id())?
            .into_iter()
            .map(FriendDetails::from)
            .collect::<Vec<_>>();
        assert_eq!(detailed_many.len(), 1);
        assert_eq!(detailed_many[0].friend.id, second.id());
        let short_many = FriendRepository::read_many_short(conn, second.id())?
            .into_iter()
            .map(FriendShort::from)
            .collect::<Vec<_>>();
        assert_eq!(short_many.len(), 1);
        assert_eq!(short_many[0].friend.id, first.id());

        assert!(FriendRepository::delete(conn, second.id(), first.id())?);
        assert!(!FriendRepository::exists(conn, first.id(), second.id())?);
        assert!(!FriendRepository::delete(conn, first.id(), second.id())?);
        Ok(())
    });
}

#[test]
fn friend_repository_reports_duplicate_and_self_friendship_constraints() {
    rollback_test(|conn| {
        let first = insert_member(conn, "friend_repo_duplicate_first")?;
        let second = insert_member(conn, "friend_repo_duplicate_second")?;
        FriendRepository::create(conn, NewFriend::new(first.id(), second.id()))?;
        assert!(matches!(
            FriendRepository::create(conn, NewFriend::new(second.id(), first.id())),
            Err(DbError::Conflict(_))
        ));
        Ok(())
    });

    rollback_test(|conn| {
        let member = insert_member(conn, "friend_repo_self")?;
        assert!(matches!(
            FriendRepository::create(conn, NewFriend::new(member.id(), member.id())),
            Err(DbError::ConstraintViolation(_))
        ));
        Ok(())
    });
}

#[test]
fn deleting_a_member_cascades_its_friendships() {
    rollback_test(|conn| {
        let first = insert_member(conn, "friend_repo_cascade_first")?;
        let second = insert_member(conn, "friend_repo_cascade_second")?;
        FriendRepository::create(conn, NewFriend::new(first.id(), second.id()))?;

        diesel::delete(member_dsl::member.filter(member_dsl::id.eq(first.id())))
            .execute(conn)
            .map_err(DbError::from)?;
        assert!(!FriendRepository::exists(conn, first.id(), second.id())?);
        Ok(())
    });
}
