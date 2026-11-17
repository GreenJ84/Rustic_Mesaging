//! Direct integration tests for friendship reads and removal.

use super::super::super::common::{client, register_user};
use crate::api::ApiError;
use crate::social::service::SocialService;
use rocket::http::Status;
use shared::models::social::request::NewFriendRequestInput;

#[test]
fn friend_service_owns_symmetric_reads_projections_and_removal() {
    let client = client();
    let first = register_user(&client, "friend_service_first");
    let second = register_user(&client, "friend_service_second");
    let service = client.rocket().state::<SocialService>().unwrap();

    service
        .send_friend_request(
            first.member.id,
            NewFriendRequestInput {
                receiver_id: second.member.id,
                note: None,
            },
        )
        .unwrap();
    service
        .accept_friend_request(second.member.id, first.member.id)
        .unwrap();

    assert!(service
        .is_friend(first.member.id, second.member.id)
        .unwrap());
    assert!(service
        .is_friend(second.member.id, first.member.id)
        .unwrap());
    assert_eq!(
        service
            .get_friend(first.member.id, second.member.id)
            .unwrap()
            .friend
            .id,
        second.member.id
    );
    assert_eq!(
        service
            .get_member_friends(first.member.id)
            .unwrap()
            .friends
            .len(),
        1
    );
    let previews = service
        .get_member_friend_previews(second.member.id)
        .unwrap();
    assert_eq!(previews.friends.len(), 1);
    assert_eq!(previews.friends[0].friend.id, first.member.id);

    service
        .remove_friend(second.member.id, first.member.id)
        .unwrap();
    assert!(!service
        .is_friend(first.member.id, second.member.id)
        .unwrap());
    assert_eq!(
        ApiError::from(
            service
                .remove_friend(first.member.id, second.member.id)
                .unwrap_err(),
        )
        .status(),
        Status::NotFound
    );
}
