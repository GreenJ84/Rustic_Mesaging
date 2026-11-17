//! HTTP integration tests for friend-request endpoints.

use super::super::super::common::{client, expect_error, json_body, register_user};
use super::super::support::send_request;
use rocket::http::{ContentType, Status};
use shared::models::social::friend::FriendDetails;
use shared::models::social::request::{FriendRequestCreation, MultiFriendRequest};

#[test]
fn request_routes_require_authentication() {
    let client = client();

    for response in [
        client
            .post("/requests")
            .header(ContentType::JSON)
            .body(r#"{"receiver_id":1,"note":null}"#)
            .dispatch(),
        client.get("/requests").dispatch(),
        client.get("/requests/1").dispatch(),
        client
            .patch("/requests/1")
            .header(ContentType::JSON)
            .body(r#"{"note":null}"#)
            .dispatch(),
        client.patch("/requests/1/accept").dispatch(),
        client.delete("/requests/1/deny").dispatch(),
        client.delete("/requests/1").dispatch(),
    ] {
        assert_eq!(response.status(), Status::Unauthorized);
    }
}

#[test]
fn request_routes_expose_creation_read_update_and_cancel_contracts() {
    let client = client();
    let sender = register_user(&client, "request_route_sender");
    let receiver = register_user(&client, "request_route_receiver");

    let response = send_request(
        &client,
        &sender,
        receiver.member.id,
        Some("  Let's connect  "),
    );
    assert_eq!(response.status(), Status::Created);
    assert!(matches!(
        response.into_json::<FriendRequestCreation>(),
        Some(FriendRequestCreation::Request(_))
    ));

    let individual = client
        .get(format!("/requests/{}", receiver.member.id))
        .header(sender.authorization())
        .dispatch();
    assert_eq!(individual.status(), Status::Ok);
    assert_eq!(
        individual.into_json::<serde_json::Value>().unwrap()["member"]["id"],
        receiver.member.id
    );

    let outgoing = client
        .get("/requests")
        .header(sender.authorization())
        .dispatch();
    assert_eq!(outgoing.status(), Status::Ok);
    assert_eq!(
        outgoing
            .into_json::<MultiFriendRequest>()
            .unwrap()
            .outgoing
            .len(),
        1
    );

    let updated = client
        .patch(format!("/requests/{}", receiver.member.id))
        .header(sender.authorization())
        .header(ContentType::JSON)
        .body(r#"{"note":"updated"}"#)
        .dispatch();
    assert_eq!(updated.status(), Status::NoContent);

    let cancelled = client
        .delete(format!("/requests/{}", receiver.member.id))
        .header(sender.authorization())
        .dispatch();
    assert_eq!(cancelled.status(), Status::NoContent);
}

#[test]
fn accept_and_deny_routes_return_the_expected_response_shapes() {
    let client = client();
    let sender = register_user(&client, "request_route_accept_sender");
    let receiver = register_user(&client, "request_route_accept_receiver");
    assert_eq!(
        send_request(&client, &sender, receiver.member.id, None).status(),
        Status::Created
    );

    let accepted = client
        .patch(format!("/requests/{}/accept", sender.member.id))
        .header(receiver.authorization())
        .dispatch();
    assert_eq!(accepted.status(), Status::Ok);
    assert_eq!(
        accepted.into_json::<FriendDetails>().unwrap().friend.id,
        sender.member.id
    );

    let second_sender = register_user(&client, "request_route_deny_sender");
    assert_eq!(
        send_request(&client, &second_sender, receiver.member.id, None).status(),
        Status::Created
    );
    let denied = client
        .delete(format!("/requests/{}/deny", second_sender.member.id))
        .header(receiver.authorization())
        .dispatch();
    assert_eq!(denied.status(), Status::NoContent);
}

#[test]
fn request_routes_reject_malformed_json_and_invalid_notes() {
    let client = client();
    let sender = register_user(&client, "request_route_invalid_sender");
    let receiver = register_user(&client, "request_route_invalid_receiver");

    expect_error(
        client
            .post("/requests")
            .header(sender.authorization())
            .header(ContentType::JSON)
            .body("{")
            .dispatch(),
        Status::BadRequest,
        "Bad request: The request itself was bad",
    );

    expect_error(
        client
            .post("/requests")
            .header(sender.authorization())
            .header(ContentType::JSON)
            .body(json_body(&serde_json::json!({
                "receiver_id": receiver.member.id,
                "note": "a".repeat(501),
            })))
            .dispatch(),
        Status::BadRequest,
        "Bad request: Friend request note is too long. Must be at most 500 characters long.",
    );
}
