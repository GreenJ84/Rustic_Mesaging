use super::common::{client, json_body, register_identity, register_user, TestIdentity};
use crate::member::models::{MemberProfile, MemberShort, MultiMemberPreview};
use rocket::http::{ContentType, Status};

#[test]
fn gets_the_authenticated_member_profile() {
    let client = client();
    let registered = register_user(&client, "member_profile");

    let response = client
        .get("/member")
        .header(registered.authorization())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let actual = response
        .into_json::<MemberProfile>()
        .expect("member profile response is JSON");
    assert_eq!(actual.id, registered.member.id);
    assert_eq!(actual.username, registered.member.username);
    assert_eq!(actual.email, registered.member.email);
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
        .into_json::<MemberProfile>()
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
fn patch_rejects_empty_and_invalid_changesets() {
    let client = client();
    let registered = register_user(&client, "member_invalid_update");

    let empty = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body("")
        .dispatch();
    assert_eq!(empty.status(), Status::BadRequest);

    let invalid_avatar = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(r#"{"avatar":"not-an-avatar"}"#)
        .dispatch();
    assert_eq!(invalid_avatar.status(), Status::BadRequest);
}

#[test]
fn patch_reports_identity_conflicts() {
    let client = client();
    let existing = register_user(&client, "existing_identity");
    let registered = register_user(&client, "conflicting_identity");
    let payload = serde_json::json!({
        "username": existing.member.username,
    });

    let response = client
        .patch("/member")
        .header(registered.authorization())
        .header(ContentType::JSON)
        .body(json_body(&payload))
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
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
            urlencoding::encode("%")
        ))
        .header(registered.authorization())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let results = response
        .into_json::<MultiMemberPreview>()
        .expect("search response is JSON");
    assert!(results
        .members
        .iter()
        .any(|member| member.username == username));
    assert!(results
        .members
        .iter()
        .all(|member| member.username.contains('%')));
}

#[test]
fn search_rejects_empty_and_oversized_terms() {
    let client = client();
    let registered = register_user(&client, "invalid_member_search");

    let empty = client
        .get("/member/search?search_term=%20%20%20")
        .header(registered.authorization())
        .dispatch();
    assert_eq!(empty.status(), Status::BadRequest);

    let oversized = client
        .get(format!("/member/search?search_term={}", "a".repeat(126)))
        .header(registered.authorization())
        .dispatch();
    assert_eq!(oversized.status(), Status::BadRequest);
}
