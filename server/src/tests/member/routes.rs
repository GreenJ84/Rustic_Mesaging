use super::super::common::{
    client, expect_error, json_body, register_identity, register_user, TestIdentity,
};
use crate::member::models::{MemberDetailed, MemberShort, MultiMemberPreview};
use rocket::http::{ContentType, Status};

#[test]
fn head_confirms_an_authenticated_member_route() {
    let client = client();
    let registered = register_user(&client, "member_head");

    let response = client
        .head("/member")
        .header(registered.authorization())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn member_root_redirects_to_the_current_member_resource() {
    let client = client();

    let response = client.get("/member").dispatch();

    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(response.headers().get_one("Location"), Some("/me"));
}

#[test]
fn protected_member_routes_reject_missing_authentication() {
    let client = client();

    for response in [
        client.head("/member").dispatch(),
        client.get("/me").dispatch(),
        client.get("/member/some-member").dispatch(),
        client
            .patch("/member")
            .header(ContentType::JSON)
            .body(r#"{"avatar":"robot"}"#)
            .dispatch(),
        client.delete("/me").dispatch(),
    ] {
        assert_eq!(response.status(), Status::Unauthorized);
    }
}

#[test]
fn gets_the_authenticated_member_profile() {
    let client = client();
    let registered = register_user(&client, "member_profile");

    let response = client
        .get("/me")
        .header(registered.authorization())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let actual = response
        .into_json::<MemberDetailed>()
        .expect("member profile response is JSON");
    assert_eq!(actual.id, registered.member.id);
    assert_eq!(actual.username, registered.member.username);
    assert_eq!(actual.email, registered.member.email);
}

#[test]
fn exact_username_lookup_returns_only_the_public_projection() {
    let client = client();
    let registered = register_user(&client, "member_lookup");
    let padded_username = format!("  {}  ", registered.member.username);
    let encoded = urlencoding::encode(&padded_username);

    let response = client
        .get(format!("/member/{encoded}"))
        .header(registered.authorization())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response
        .into_json::<serde_json::Value>()
        .expect("member preview response is JSON");
    assert_eq!(body["id"], registered.member.id);
    assert_eq!(body["username"], registered.member.username);
    assert!(body.get("avatar").is_some());
    assert!(body.get("email").is_none());
    assert!(body.get("password_hash").is_none());
}

#[test]
fn exact_username_lookup_rejects_invalid_names() {
    let client = client();
    let registered = register_user(&client, "member_lookup_invalid");

    let response = client
        .get("/member/ab")
        .header(registered.authorization())
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: Username is too short. Must be at least 3 characters long.",
    );
}

#[test]
fn exact_username_lookup_reports_missing_members() {
    let client = client();
    let registered = register_user(&client, "member_lookup_missing");
    let missing = TestIdentity::unique("missing_member").username;

    let response = client
        .get(format!("/member/{missing}"))
        .header(registered.authorization())
        .dispatch();

    expect_error(response, Status::NotFound, "Not found: Member not found");
}

#[test]
fn patch_normalizes_and_returns_the_updated_profile() {
    let client = client();
    let registered = register_user(&client, "member_update");
    let updated_username = TestIdentity::unique("UpdatedMember").username;
    let payload = serde_json::json!({
        "username": format!("  {updated_username}  "),
        "avatar": "  robot  ",
    });

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(json_body(&payload))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let updated = response
        .into_json::<MemberDetailed>()
        .expect("updated profile response is JSON");
    assert_eq!(updated.username, updated_username);
    assert_eq!(updated.avatar, "robot");

    let lookup = client
        .get(format!("/member/{updated_username}"))
        .header(registered.authorization())
        .dispatch();
    assert_eq!(lookup.status(), Status::Ok);
    let preview = lookup
        .into_json::<MemberShort>()
        .expect("member preview response is JSON");
    assert_eq!(preview.username, updated_username);
}

#[test]
fn patch_allows_reusing_the_members_own_identity() {
    let client = client();
    let registered = register_user(&client, "member_same_identity");
    let payload = serde_json::json!({
        "username": registered.member.username,
        "email": registered.member.email,
    });

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(json_body(&payload))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn patch_rejects_malformed_json() {
    let client = client();
    let registered = register_user(&client, "member_malformed_update");

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body("{")
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: The request itself was bad",
    );
}

#[test]
fn patch_rejects_an_empty_changeset() {
    let client = client();
    let registered = register_user(&client, "member_empty_update");

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body("{}")
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: No changes provided",
    );
}

#[test]
fn patch_rejects_an_invalid_username() {
    let client = client();
    let registered = register_user(&client, "member_invalid_username");

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(r#"{"username":"ab"}"#)
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: Username is too short. Must be at least 3 characters long.",
    );
}

#[test]
fn patch_rejects_an_invalid_email() {
    let client = client();
    let registered = register_user(&client, "member_invalid_email");

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(r#"{"email":"not-an-email"}"#)
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: Email must contain '@'",
    );
}

#[test]
fn patch_rejects_an_invalid_avatar() {
    let client = client();
    let registered = register_user(&client, "member_invalid_avatar");

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(r#"{"avatar":"not-an-avatar"}"#)
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: Invalid avatar value",
    );
}

#[test]
fn patch_reports_username_conflicts() {
    let client = client();
    let existing = register_user(&client, "existing_username");
    let registered = register_user(&client, "conflicting_username");
    let payload = serde_json::json!({
        "username": existing.member.username,
    });

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(json_body(&payload))
        .dispatch();

    expect_error(
        response,
        Status::Conflict,
        "Conflict: Username is already taken",
    );
}

#[test]
fn patch_reports_email_conflicts() {
    let client = client();
    let existing = register_user(&client, "existing_email");
    let registered = register_user(&client, "conflicting_email");
    let payload = serde_json::json!({
        "email": existing.member.email,
    });

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(json_body(&payload))
        .dispatch();

    expect_error(
        response,
        Status::Conflict,
        "Conflict: Email is already taken",
    );
}

#[test]
fn search_is_public_and_returns_short_member_contracts() {
    let client = client();
    let identity = TestIdentity::unique("public_search_member");
    let username = identity.username.clone();
    register_identity(&client, identity);

    let response = client
        .get(format!(
            "/member/search?search_term={}",
            urlencoding::encode(&username)
        ))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body = response
        .into_json::<serde_json::Value>()
        .expect("search response is JSON");
    let member = body["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|member| member["username"] == username)
        .expect("registered member should be returned");
    assert!(member.get("avatar").is_some());
    assert!(member.get("email").is_none());
}

#[test]
fn search_treats_sql_wildcards_as_literal_characters() {
    let client = client();
    let identity = TestIdentity::unique("literal%member");
    let username = identity.username.clone();
    let registered = register_identity(&client, identity);
    let _ordinary = register_user(&client, "ordinary_search_member");

    let response = client
        .get(format!(
            "/member/search?search_term={}",
            urlencoding::encode(&username)
        ))
        .header(registered.authorization())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let results = response
        .into_json::<MultiMemberPreview>()
        .expect("search response is JSON");
    assert_eq!(results.members.len(), 1);
    assert_eq!(results.members[0].username, username);
    assert!(results
        .members
        .iter()
        .all(|member| member.username.contains('%')));
}

#[test]
fn search_rejects_empty_terms() {
    let client = client();

    let response = client
        .get("/member/search?search_term=%20%20%20")
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: Members search term is empty.",
    );
}

#[test]
fn search_rejects_oversized_terms() {
    let client = client();

    let response = client
        .get(format!("/member/search?search_term={}", "a".repeat(101)))
        .dispatch();

    expect_error(
        response,
        Status::BadRequest,
        "Bad request: Members search term is too long. Must be at most 100 characters long.",
    );
}

#[test]
fn deleting_an_ordinary_member_removes_the_account_and_session() {
    let client = client();
    let registered = register_user(&client, "member_delete");

    let response = client
        .delete("/me")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(json_body(&registered.identity.registration_payload()))
        .dispatch();

    assert_eq!(response.status(), Status::NoContent);

    let protected = client
        .get("/me")
        .header(registered.authorization())
        .dispatch();
    assert_eq!(protected.status(), Status::Unauthorized);
}

#[test]
fn deletion_requires_the_current_identity_and_password_confirmation() {
    let client = client();
    let registered = register_user(&client, "member_delete_confirmation");
    let mut wrong_identity = registered.identity.registration_payload();
    wrong_identity["username"] = serde_json::json!("someone_else");

    expect_error(
        client
            .delete("/me")
            .header(registered.authorization())
            .header(ContentType::JSON)
            .body(json_body(&wrong_identity))
            .dispatch(),
        Status::Unauthorized,
        "Unauthorized: Invalid username or password.",
    );

    let mut mismatched_password = registered.identity.registration_payload();
    mismatched_password["confirm_password"] = serde_json::json!("different-password");
    expect_error(
        client
            .delete("/me")
            .header(registered.authorization())
            .header(ContentType::JSON)
            .body(json_body(&mismatched_password))
            .dispatch(),
        Status::BadRequest,
        "Bad request: Password confirmation does not match.",
    );
}

#[test]
fn deletion_is_blocked_while_the_member_owns_a_server() {
    let client = client();
    let registered = register_user(&client, "member_delete_server_owner");
    let server_name = format!("owned_{}", uuid::Uuid::now_v7().simple());
    let created = client
        .post("/servers")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(json_body(&serde_json::json!({
            "name": server_name,
            "description": "A server that prevents accidental account deletion",
            "icon": null,
        })))
        .dispatch();
    assert_eq!(created.status(), Status::Created);

    expect_error(
        client
            .delete("/me")
            .header(registered.authorization())
            .header(ContentType::JSON)
            .body(json_body(&registered.identity.registration_payload()))
            .dispatch(),
        Status::Conflict,
        "Conflict: Transfer or delete owned servers before deleting the account",
    );
}

#[test]
fn deletion_is_blocked_while_the_member_owns_a_chat() {
    let client = client();
    let registered = register_user(&client, "member_delete_chat_owner");
    let created = client
        .post("/chat")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(r#"{"name":"Owned room","member_ids":[],"group":true}"#)
        .dispatch();
    assert_eq!(created.status(), Status::Created);

    expect_error(
        client
            .delete("/me")
            .header(registered.authorization())
            .header(ContentType::JSON)
            .body(json_body(&registered.identity.registration_payload()))
            .dispatch(),
        Status::Conflict,
        "Conflict: Delete owned chats before deleting the account",
    );
}
