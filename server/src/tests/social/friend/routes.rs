//! HTTP integration tests for friendship endpoints.

use super::super::super::common::{client, expect_error, register_user};
use super::super::support::create_friendship;
use rocket::http::Status;
use shared::models::social::friend::{FriendDetails, MultiFriend};

#[test]
fn friend_routes_require_authentication() {
    let client = client();

    for response in [
        client.get("/friends").dispatch(),
        client.get("/friends/detailed").dispatch(),
        client.get("/friends/1").dispatch(),
        client.get("/friends/1/status").dispatch(),
        client.delete("/friends/1").dispatch(),
    ] {
        assert_eq!(response.status(), Status::Unauthorized);
    }
}

#[test]
fn friend_routes_return_preview_detailed_status_and_delete_contracts() {
    let client = client();
    let first = register_user(&client, "friend_route_first");
    let second = register_user(&client, "friend_route_second");
    create_friendship(&client, &first, &second);

    let individual = client
        .get(format!("/friends/{}", second.member.id))
        .header(first.authorization())
        .dispatch();
    assert_eq!(individual.status(), Status::Ok);
    assert_eq!(
        individual.into_json::<FriendDetails>().unwrap().friend.id,
        second.member.id
    );

    let previews = client
        .get("/friends")
        .header(first.authorization())
        .dispatch();
    assert_eq!(previews.status(), Status::Ok);
    let preview = previews.into_json::<serde_json::Value>().unwrap();
    assert_eq!(preview["friends"][0]["friend"]["id"], second.member.id);
    assert!(preview["friends"][0]["friend"].get("email").is_none());

    let detailed = client
        .get("/friends/detailed")
        .header(first.authorization())
        .dispatch();
    assert_eq!(detailed.status(), Status::Ok);
    assert_eq!(
        detailed.into_json::<MultiFriend>().unwrap().friends[0]
            .friend
            .id,
        second.member.id
    );

    let status = client
        .get(format!("/friends/{}/status", second.member.id))
        .header(first.authorization())
        .dispatch();
    assert_eq!(status.into_json::<bool>(), Some(true));

    let removed = client
        .delete(format!("/friends/{}", first.member.id))
        .header(second.authorization())
        .dispatch();
    assert_eq!(removed.status(), Status::NoContent);
    expect_error(
        client
            .delete(format!("/friends/{}", second.member.id))
            .header(first.authorization())
            .dispatch(),
        Status::NotFound,
        "Not found: Friendship not found",
    );
}
