//! Rollback-isolated persistence tests for friend requests.

use super::models::{FriendRequestChangeset, NewFriendRequest};
use super::repository::{FriendRequestDirection, FriendRequestRepository};
use crate::infra::DbError;
use crate::schema::member::dsl as member_dsl;
use crate::tests::social::support::{insert_member, rollback_test};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use shared::models::social::request::{FriendRequest, FriendRequestDetailed};

#[test]
fn request_repository_covers_direction_projections_update_and_delete() {
    rollback_test(|conn| {
        let sender = insert_member(conn, "request_repo_sender")?;
        let receiver = insert_member(conn, "request_repo_receiver")?;
        let created = FriendRequestRepository::create(
            conn,
            NewFriendRequest {
                sender_id: sender.id(),
                receiver_id: receiver.id(),
                note: Some("initial".into()),
            },
        )?;
        let created = FriendRequest::from(created);
        assert_eq!(created.sender_id, sender.id());
        assert_eq!(created.receiver_id, receiver.id());

        assert!(FriendRequestRepository::read(conn, sender.id(), receiver.id())?.is_some());
        assert!(matches!(
            FriendRequestRepository::read_direction_for_update(conn, sender.id(), receiver.id())?,
            Some(FriendRequestDirection::Outgoing)
        ));
        assert!(matches!(
            FriendRequestRepository::read_direction_for_update(conn, receiver.id(), sender.id())?,
            Some(FriendRequestDirection::Incoming)
        ));

        let detailed = FriendRequestRepository::read_detailed(conn, sender.id(), receiver.id())?
            .map(FriendRequestDetailed::from)
            .expect("request should have a detailed projection");
        assert_eq!(detailed.member.id, receiver.id());
        assert_eq!(
            FriendRequestRepository::read_outgoing(conn, sender.id())?.len(),
            1
        );
        assert_eq!(
            FriendRequestRepository::read_incoming(conn, receiver.id())?.len(),
            1
        );

        let updated = FriendRequestRepository::update(
            conn,
            sender.id(),
            receiver.id(),
            FriendRequestChangeset { note: None },
        )?
        .expect("existing request should update");
        assert!(updated.note().is_none());
        assert!(FriendRequestRepository::delete(
            conn,
            sender.id(),
            receiver.id()
        )?);
        assert!(FriendRequestRepository::read(conn, sender.id(), receiver.id())?.is_none());
        assert!(!FriendRequestRepository::delete(
            conn,
            sender.id(),
            receiver.id()
        )?);
        Ok(())
    });
}

#[test]
fn request_repository_enforces_unordered_pair_and_self_constraints() {
    rollback_test(|conn| {
        let first = insert_member(conn, "request_repo_pair_first")?;
        let second = insert_member(conn, "request_repo_pair_second")?;
        FriendRequestRepository::create(
            conn,
            NewFriendRequest {
                sender_id: first.id(),
                receiver_id: second.id(),
                note: None,
            },
        )?;
        assert!(matches!(
            FriendRequestRepository::create(
                conn,
                NewFriendRequest {
                    sender_id: second.id(),
                    receiver_id: first.id(),
                    note: None,
                }
            ),
            Err(DbError::Conflict(_))
        ));
        Ok(())
    });

    rollback_test(|conn| {
        let member = insert_member(conn, "request_repo_self")?;
        assert!(matches!(
            FriendRequestRepository::create(
                conn,
                NewFriendRequest {
                    sender_id: member.id(),
                    receiver_id: member.id(),
                    note: None,
                }
            ),
            Err(DbError::ConstraintViolation(_))
        ));
        Ok(())
    });
}

#[test]
fn deleting_a_member_cascades_its_pending_requests() {
    rollback_test(|conn| {
        let sender = insert_member(conn, "request_repo_cascade_sender")?;
        let receiver = insert_member(conn, "request_repo_cascade_receiver")?;
        FriendRequestRepository::create(
            conn,
            NewFriendRequest {
                sender_id: sender.id(),
                receiver_id: receiver.id(),
                note: None,
            },
        )?;

        diesel::delete(member_dsl::member.filter(member_dsl::id.eq(receiver.id())))
            .execute(conn)
            .map_err(DbError::from)?;
        assert!(FriendRequestRepository::read(conn, sender.id(), receiver.id())?.is_none());
        Ok(())
    });
}
