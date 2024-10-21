use rocket::local::blocking::Client;
use rocket::http::{ContentType, Status};
use rocket::{Ignite, Rocket};
use rocket::serde::json::json;
use crate::models::authentication::AuthenticationResponse;
use crate::models::member::MemberSafe;
use super::rocket;


#[test]
fn prevent_unregistered_user() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get(uri!("/member")).dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
    assert_eq!(response.into_string().unwrap(), "Unauthorized: Error validating member authentication")
}

#[test]
fn register_user() {
    let form_data = vec![
        ("username", "not_a_used_username"),
        ("password", "password123"),
        ("confirm_password", "password123"),
        ("email", "new_user@example.com")
    ];
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let mut response = client.post("/register")
        .header(ContentType::Form)
        .body(form_data.iter()
            .map(|(key, value)| format!("{}={}", key.to_string(), urlencoding::encode(value).to_string()))
            .collect::<Vec<String>>()
            .join("&")
        )
        .dispatch();
    assert_eq!(response.status(), Status::Created);
    let response_json = response.into_json::<MemberSafe>();
    assert!(response_json.is_some());
}

#[test]
fn prevent_duplicate_username_registration() {
    let form_data = vec![
        ("username", "twice_used_username"),
        ("password", "password123"),
        ("confirm_password", "password123"),
        ("email", "twice_used_username@example.com")
    ];
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let mut response = client.post("/register")
        .header(ContentType::Form)
        .body(form_data.iter()
            .map(|(key, value)| format!("{}={}", key.to_string(), urlencoding::encode(value).to_string()))
            .collect::<Vec<String>>()
            .join("&")
        )
        .dispatch();
    assert_eq!(response.status(), Status::Created);

    response = client.post("/register")
        .header(ContentType::Form)
        .body(form_data.iter()
            .map(|(key, value)| format!("{}={}", key.to_string(), urlencoding::encode(value).to_string()))
            .collect::<Vec<String>>()
            .join("&")
        )
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
    assert_eq!(response.into_string().unwrap(), "User already exists.")
}

#[test]
fn login_registered_user() {
    let registered_user_credentials = vec![
        ("username", "GreenJ"),
        ("password", "password123"),
    ];
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let mut response = client.post("/login")
        .header(ContentType::Form)
        .body(registered_user_credentials.iter()
            .map(|(key, value)| format!("{}={}", key.to_string(), urlencoding::encode(value).to_string()))
            .collect::<Vec<String>>()
            .join("&")
        )
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let response_json = response.into_json::<MemberSafe>();
    assert!(response_json.is_some());
}
