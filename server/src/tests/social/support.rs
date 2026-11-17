//! Shared fixtures for Social integration tests.

use diesel::Connection;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::{Client, LocalResponse};

use super::super::common::{database_connection, json_body, TestSession};
use crate::infra::{DbConn, DbError, DieselError};
use crate::member::models::{MemberRow, NewMember};
use crate::member::repository::MemberRepository;
use shared::models::social::friend::FriendDetails;

pub(super) fn send_request<'client>(
    client: &'client Client,
    sender: &TestSession,
    receiver_id: i32,
    note: Option<&str>,
) -> LocalResponse<'client> {
    client
        .post("/requests")
        .header(sender.authorization())
        .header(ContentType::JSON)
        .body(json_body(&serde_json::json!({
            "receiver_id": receiver_id,
            "note": note,
        })))
        .dispatch()
}

pub(super) fn create_friendship(
    client: &Client,
    sender: &TestSession,
    receiver: &TestSession,
) -> FriendDetails {
    assert_eq!(
        send_request(client, sender, receiver.member.id, None).status(),
        Status::Created
    );
    let response = client
        .patch(format!("/requests/{}/accept", sender.member.id))
        .header(receiver.authorization())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    response
        .into_json()
        .expect("accepted friendship response should be JSON")
}

pub(crate) fn unique(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::now_v7().simple())
}

pub(crate) fn insert_member(conn: &mut DbConn, prefix: &str) -> Result<MemberRow, DbError> {
    let username = unique(prefix);
    MemberRepository::create(
        conn,
        NewMember::new(
            username.clone(),
            "social-repository-test-hash".into(),
            format!("{username}@example.com"),
        ),
    )
}

pub(crate) fn rollback_test(mut test: impl FnMut(&mut DbConn) -> Result<(), DbError>) {
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
            Err(error) => panic!("social repository test transaction failed: {error:?}"),
            Ok(()) => unreachable!("repository test transaction always rolls back"),
        }
    }
}
