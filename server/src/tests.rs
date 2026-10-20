use super::rocket;
use crate::member::models::MemberProfile;
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;

const TEST_PASSWORD: &str = "correct-horse-123";

fn form_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={}", urlencoding::encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn unique_identity(prefix: &str) -> (String, String) {
    let suffix = uuid::Uuid::now_v7().simple().to_string();
    (
        format!("{prefix}_{suffix}"),
        format!("{prefix}_{suffix}@example.com"),
    )
}

fn register_and_get_auth(client: &Client, prefix: &str) -> (String, MemberProfile) {
    let (username, email) = unique_identity(prefix);
    let fields = [
        ("username", username.as_str()),
        ("password", TEST_PASSWORD),
        ("confirm_password", TEST_PASSWORD),
        ("email", email.as_str()),
    ];
    let response = client
        .post("/register")
        .header(ContentType::Form)
        .body(form_body(&fields))
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    let token = response
        .headers()
        .get_one("Authorization")
        .expect("registration returns an access token")
        .to_owned();
    let member = response
        .into_json::<MemberProfile>()
        .expect("registration returns the member");
    (token, member)
}

fn register_and_get_access_token(client: &Client, prefix: &str) -> String {
    register_and_get_auth(client, prefix).0
}

fn admin_access_token(client: &Client) -> String {
    let fields = [("username", "GreenJ"), ("password", "password123")];
    let response = client
        .post("/login")
        .header(ContentType::Form)
        .body(form_body(&fields))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    response
        .headers()
        .get_one("Authorization")
        .expect("administrator login returns an access token")
        .to_owned()
}

#[test]
fn prevent_unregistered_user() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get(uri!("/member")).dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn register_user() {
    let (username, email) = unique_identity("new_user");
    let fields = [
        ("username", username.as_str()),
        ("password", TEST_PASSWORD),
        ("confirm_password", TEST_PASSWORD),
        ("email", email.as_str()),
    ];
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client
        .post("/register")
        .header(ContentType::Form)
        .body(form_body(&fields))
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert!(response.into_json::<MemberProfile>().is_some());
}

#[test]
fn prevent_duplicate_username_registration() {
    let (username, email) = unique_identity("duplicate_user");
    let fields = [
        ("username", username.as_str()),
        ("password", TEST_PASSWORD),
        ("confirm_password", TEST_PASSWORD),
        ("email", email.as_str()),
    ];
    let client = Client::tracked(rocket()).expect("valid rocket instance");

    let first = client
        .post("/register")
        .header(ContentType::Form)
        .body(form_body(&fields))
        .dispatch();
    assert_eq!(first.status(), Status::Created);

    let duplicate = client
        .post("/register")
        .header(ContentType::Form)
        .body(form_body(&fields))
        .dispatch();
    assert_eq!(duplicate.status(), Status::Conflict);
    assert!(duplicate.into_json::<serde_json::Value>().is_some());
}

#[test]
fn reports_duplicate_email_registration() {
    let (first_username, email) = unique_identity("duplicate_email");
    let (second_username, _) = unique_identity("other_username");
    let client = Client::tracked(rocket()).expect("valid rocket instance");

    for (username, expected_status) in [
        (first_username.as_str(), Status::Created),
        (second_username.as_str(), Status::Conflict),
    ] {
        let fields = [
            ("username", username),
            ("password", TEST_PASSWORD),
            ("confirm_password", TEST_PASSWORD),
            ("email", email.as_str()),
        ];
        let response = client
            .post("/register")
            .header(ContentType::Form)
            .body(form_body(&fields))
            .dispatch();
        assert_eq!(response.status(), expected_status);

        if expected_status == Status::Conflict {
            let body = response
                .into_json::<serde_json::Value>()
                .expect("conflicts return a JSON error");
            assert_eq!(body["message"], "Email is already taken.");
        }
    }
}

#[test]
fn refresh_rotates_session_and_logout_revokes_replacement() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let (original_token, member) = register_and_get_auth(&client, "rotation_user");

    let refresh = client.post(format!("/refresh/{}", member.id)).dispatch();
    assert_eq!(refresh.status(), Status::Ok);
    let replacement_token = refresh
        .headers()
        .get_one("Authorization")
        .expect("refresh returns a replacement access token")
        .to_owned();

    let stale = client
        .get("/member")
        .header(Header::new("Authorization", original_token))
        .dispatch();
    assert_eq!(stale.status(), Status::Unauthorized);

    let current = client
        .get("/member")
        .header(Header::new("Authorization", replacement_token.clone()))
        .dispatch();
    assert_eq!(current.status(), Status::Ok);

    let logout = client
        .post("/logout")
        .header(Header::new("Authorization", replacement_token.clone()))
        .dispatch();
    assert_eq!(logout.status(), Status::NoContent);

    let revoked = client
        .get("/member")
        .header(Header::new("Authorization", replacement_token))
        .dispatch();
    assert_eq!(revoked.status(), Status::Unauthorized);
}

#[test]
fn refresh_rejects_member_mismatch_without_consuming_token() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let (_, other_member) = register_and_get_auth(&client, "other_refresh_user");
    let (_, token_owner) = register_and_get_auth(&client, "refresh_token_owner");
    let refresh_cookie = client
        .cookies()
        .get("refresh_token")
        .expect("registration sets a refresh cookie")
        .clone();

    let mismatch = client
        .post(format!("/refresh/{}", other_member.id))
        .dispatch();
    assert_eq!(mismatch.status(), Status::Unauthorized);

    let valid = client
        .post(format!("/refresh/{}", token_owner.id))
        .cookie(refresh_cookie)
        .dispatch();
    assert_eq!(valid.status(), Status::Ok);
}

#[test]
fn ordinary_users_cannot_manage_system_roles() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let token = register_and_get_access_token(&client, "role_denied_user");
    let response = client
        .get("/roles/")
        .header(Header::new("Authorization", token))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
fn role_changes_drive_authorization_and_legacy_admin_state() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let admin_token = admin_access_token(&client);
    let (member_token, member) = register_and_get_auth(&client, "promoted_user");

    let roles = client
        .get("/roles/")
        .header(Header::new("Authorization", admin_token.clone()))
        .dispatch();
    assert_eq!(roles.status(), Status::Ok);
    let roles = roles
        .into_json::<Vec<serde_json::Value>>()
        .expect("role listing is JSON");
    let admin_role_id = roles
        .iter()
        .find(|role| role["name"] == "admin")
        .and_then(|role| role["id"].as_i64())
        .expect("admin role is configured");

    let promoted = client
        .put(format!("/roles/{admin_role_id}/members/{}", member.id))
        .header(Header::new("Authorization", admin_token.clone()))
        .dispatch();
    assert_eq!(promoted.status(), Status::NoContent);

    let promoted_member = client
        .get("/member")
        .header(Header::new("Authorization", member_token.clone()))
        .dispatch()
        .into_json::<MemberProfile>()
        .expect("promoted member remains authenticated");

    let now_authorized = client
        .get("/roles/")
        .header(Header::new("Authorization", member_token.clone()))
        .dispatch();
    assert_eq!(now_authorized.status(), Status::Ok);

    let reset = client
        .delete(format!("/roles/members/{}", member.id))
        .header(Header::new("Authorization", admin_token))
        .dispatch();
    assert_eq!(reset.status(), Status::NoContent);

    let reset_member = client
        .get("/member")
        .header(Header::new("Authorization", member_token.clone()))
        .dispatch()
        .into_json::<MemberProfile>()
        .expect("reset member remains authenticated");

    let no_longer_authorized = client
        .get("/roles/")
        .header(Header::new("Authorization", member_token))
        .dispatch();
    assert_eq!(no_longer_authorized.status(), Status::Forbidden);
}

#[test]
fn login_registered_user() {
    let fields = [("username", "GreenJ"), ("password", "password123")];
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client
        .post("/login")
        .header(ContentType::Form)
        .body(form_body(&fields))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response.into_json::<MemberProfile>().is_some());
}
