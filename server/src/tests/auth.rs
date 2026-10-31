use super::common::{
    client, json_body, register_admin, register_user as register_test_user, TestIdentity,
    TEST_PASSWORD, login
};
use crate::member::models::MemberProfile;
use rocket::http::{ContentType, Header, Status};

#[test]
fn prevent_unregistered_user() {
    let client = client();
    let response = client.get(uri!("/member")).dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn register_user() {
    let identity = TestIdentity::unique("new_user");
    let client = client();
    let response = client
        .post("/register")
        .header(ContentType::JSON)
        .body(json_body(&identity.registration_payload()))
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert!(response.into_json::<MemberProfile>().is_some());
}

#[test]
fn prevent_duplicate_username_registration() {
    let identity = TestIdentity::unique("duplicate_user");
    let client = client();

    let first = client
        .post("/register")
        .header(ContentType::JSON)
        .body(json_body(&identity.registration_payload()))
        .dispatch();
    assert_eq!(first.status(), Status::Created);

    let duplicate = client
        .post("/register")
        .header(ContentType::JSON)
        .body(json_body(&identity.registration_payload()))
        .dispatch();
    assert_eq!(duplicate.status(), Status::Conflict);
    assert!(duplicate.into_json::<serde_json::Value>().is_some());
}

#[test]
fn reports_duplicate_email_registration() {
    let first = TestIdentity::unique("duplicate_email");
    let second = TestIdentity::unique("other_username");
    let client = client();

    for (username, expected_status) in [
        (first.username.as_str(), Status::Created),
        (second.username.as_str(), Status::Conflict),
    ] {
        let payload = serde_json::json!({
            "username": username,
            "password": TEST_PASSWORD,
            "confirm_password": TEST_PASSWORD,
            "email": first.email,
        });
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(json_body(&payload))
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
    let client = client();
    let registered = register_test_user(&client, "rotation_user");

    let refresh = client
        .post(format!("/refresh/{}", registered.member.id))
        .dispatch();
    assert_eq!(refresh.status(), Status::Ok);
    let replacement_token = refresh
        .headers()
        .get_one("Authorization")
        .expect("refresh returns a replacement access token")
        .to_owned();

    let stale = client
        .get("/member")
        .header(registered.authorization())
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
    let client = client();
    let other_member = register_test_user(&client, "other_refresh_user");
    let token_owner = register_test_user(&client, "refresh_token_owner");
    let refresh_cookie = client
        .cookies()
        .get("refresh_token")
        .expect("registration sets a refresh cookie")
        .clone();

    let mismatch = client
        .post(format!("/refresh/{}", other_member.member.id))
        .dispatch();
    assert_eq!(mismatch.status(), Status::Unauthorized);

    let valid = client
        .post(format!("/refresh/{}", token_owner.member.id))
        .cookie(refresh_cookie)
        .dispatch();
    assert_eq!(valid.status(), Status::Ok);
}

#[test]
fn ordinary_users_cannot_manage_system_roles() {
    let client = client();
    let token = register_test_user(&client, "role_denied_user").access_token;
    let response = client
        .get("/roles/")
        .header(Header::new("Authorization", token))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
fn role_changes_drive_authorization_and_legacy_admin_state() {
    let client = client();
    let admin_token = register_admin(&client, "role_admin").access_token;
    let member = register_test_user(&client, "promoted_user");

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
        .put(format!(
            "/roles/{admin_role_id}/members/{}",
            member.member.id
        ))
        .header(Header::new("Authorization", admin_token.clone()))
        .dispatch();
    assert_eq!(promoted.status(), Status::NoContent);

    let _promoted_member = client
        .get("/member")
        .header(member.authorization())
        .dispatch()
        .into_json::<MemberProfile>()
        .expect("promoted member remains authenticated");

    let now_authorized = client
        .get("/roles/")
        .header(member.authorization())
        .dispatch();
    assert_eq!(now_authorized.status(), Status::Ok);

    let reset = client
        .delete(format!("/roles/members/{}", member.member.id))
        .header(Header::new("Authorization", admin_token))
        .dispatch();
    assert_eq!(reset.status(), Status::NoContent);

    let _reset_member = client
        .get("/member")
        .header(member.authorization())
        .dispatch()
        .into_json::<MemberProfile>()
        .expect("reset member remains authenticated");

    let no_longer_authorized = client
        .get("/roles/")
        .header(member.authorization())
        .dispatch();
    assert_eq!(no_longer_authorized.status(), Status::Forbidden);
}

#[test]
fn login_registered_user() {
    let registration_client = client();
    let registered = register_test_user(&registration_client, "login_user");
    let login_client = client();
    let authenticated = login(&login_client, &registered.identity);

    assert_eq!(authenticated.member.id, registered.member.id);
}
