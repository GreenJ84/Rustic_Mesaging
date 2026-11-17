//! Direct integration tests for friend-request orchestration and policy.

use super::super::super::common::{client, register_user};
use crate::api::ApiError;
use crate::social::service::SocialService;
use rocket::http::Status;
use shared::models::social::request::{
    FriendRequestChangesetInput, FriendRequestCreation, NewFriendRequestInput,
};

#[test]
fn request_service_owns_creation_updates_direction_and_acceptance() {
    let client = client();
    let sender = register_user(&client, "request_service_sender");
    let receiver = register_user(&client, "request_service_receiver");
    let service = client.rocket().state::<SocialService>().unwrap();

    let created = service
        .send_friend_request(
            sender.member.id,
            NewFriendRequestInput {
                receiver_id: receiver.member.id,
                note: Some("  Let's connect  ".into()),
            },
        )
        .unwrap();
    match created {
        FriendRequestCreation::Request(request) => {
            assert_eq!(request.member.id, receiver.member.id);
            assert_eq!(request.note.as_deref(), Some("Let's connect"));
        }
        FriendRequestCreation::Friend(_) => panic!("first request must remain pending"),
    }

    let sender_requests = service.get_friend_requests(sender.member.id).unwrap();
    assert!(sender_requests.incoming.is_empty());
    assert_eq!(sender_requests.outgoing.len(), 1);
    let receiver_requests = service.get_friend_requests(receiver.member.id).unwrap();
    assert_eq!(receiver_requests.incoming.len(), 1);
    assert!(receiver_requests.outgoing.is_empty());

    assert_eq!(
        ApiError::from(
            service
                .update_request(
                    receiver.member.id,
                    sender.member.id,
                    FriendRequestChangesetInput {
                        note: Some("forbidden".into()),
                    },
                )
                .unwrap_err(),
        )
        .status(),
        Status::Forbidden
    );
    service
        .update_request(
            sender.member.id,
            receiver.member.id,
            FriendRequestChangesetInput { note: None },
        )
        .unwrap();
    assert!(service
        .get_friend_request(sender.member.id, receiver.member.id)
        .unwrap()
        .note
        .is_none());

    assert_eq!(
        ApiError::from(
            service
                .accept_friend_request(sender.member.id, receiver.member.id)
                .unwrap_err(),
        )
        .status(),
        Status::Forbidden
    );
    let friendship = service
        .accept_friend_request(receiver.member.id, sender.member.id)
        .unwrap();
    assert_eq!(friendship.friend.id, sender.member.id);
    assert!(service
        .get_friend_requests(sender.member.id)
        .unwrap()
        .outgoing
        .is_empty());
    assert_eq!(
        ApiError::from(
            service
                .send_friend_request(
                    sender.member.id,
                    NewFriendRequestInput {
                        receiver_id: receiver.member.id,
                        note: None,
                    },
                )
                .unwrap_err(),
        )
        .status(),
        Status::Conflict
    );
}

#[test]
fn request_service_enforces_cancel_and_deny_direction() {
    let client = client();
    let sender = register_user(&client, "request_service_cancel_sender");
    let receiver = register_user(&client, "request_service_cancel_receiver");
    let service = client.rocket().state::<SocialService>().unwrap();

    service
        .send_friend_request(
            sender.member.id,
            NewFriendRequestInput {
                receiver_id: receiver.member.id,
                note: None,
            },
        )
        .unwrap();
    assert_eq!(
        ApiError::from(
            service
                .cancel_friend_request(receiver.member.id, sender.member.id)
                .unwrap_err(),
        )
        .status(),
        Status::Forbidden
    );
    service
        .cancel_friend_request(sender.member.id, receiver.member.id)
        .unwrap();

    service
        .send_friend_request(
            sender.member.id,
            NewFriendRequestInput {
                receiver_id: receiver.member.id,
                note: None,
            },
        )
        .unwrap();
    assert_eq!(
        ApiError::from(
            service
                .deny_friend_request(sender.member.id, receiver.member.id)
                .unwrap_err(),
        )
        .status(),
        Status::Forbidden
    );
    service
        .deny_friend_request(receiver.member.id, sender.member.id)
        .unwrap();
    assert_eq!(
        ApiError::from(
            service
                .get_friend_request(sender.member.id, receiver.member.id)
                .unwrap_err(),
        )
        .status(),
        Status::NotFound
    );
}

#[test]
fn request_service_validates_targets_duplicates_and_inverse_requests() {
    let client = client();
    let first = register_user(&client, "request_service_inverse_first");
    let second = register_user(&client, "request_service_inverse_second");
    let service = client.rocket().state::<SocialService>().unwrap();

    assert_eq!(
        ApiError::from(
            service
                .send_friend_request(
                    first.member.id,
                    NewFriendRequestInput {
                        receiver_id: first.member.id,
                        note: None,
                    },
                )
                .unwrap_err(),
        )
        .status(),
        Status::BadRequest
    );
    assert_eq!(
        ApiError::from(
            service
                .send_friend_request(
                    first.member.id,
                    NewFriendRequestInput {
                        receiver_id: i32::MAX,
                        note: None,
                    },
                )
                .unwrap_err(),
        )
        .status(),
        Status::NotFound
    );

    service
        .send_friend_request(
            first.member.id,
            NewFriendRequestInput {
                receiver_id: second.member.id,
                note: None,
            },
        )
        .unwrap();
    assert_eq!(
        ApiError::from(
            service
                .send_friend_request(
                    first.member.id,
                    NewFriendRequestInput {
                        receiver_id: second.member.id,
                        note: None,
                    },
                )
                .unwrap_err(),
        )
        .status(),
        Status::Conflict
    );

    assert!(matches!(
        service
            .send_friend_request(
                second.member.id,
                NewFriendRequestInput {
                    receiver_id: first.member.id,
                    note: None,
                },
            )
            .unwrap(),
        FriendRequestCreation::Friend(_)
    ));
}
