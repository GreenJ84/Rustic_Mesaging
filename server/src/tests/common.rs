use std::env;

use crate::member::models::MemberProfile;
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::{Client, LocalResponse};
use serde::{Deserialize, Serialize};

pub(super) const TEST_PASSWORD: &str = "correct-horse-123";
pub(super) const TEST_NEW_PASSWORD: &str = "even-better-horse-456";

const ADMINISTRATION_ENV: &str = "ADMINISTRATION_KEY";
const ADMINISTRATION_HEADER: &str = "Administration-Key";

#[derive(Debug, Deserialize)]
pub(super) struct TestApiError {
    pub code: String,
    pub message: String,
}

#[derive(Clone)]
pub(super) struct TestIdentity {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl TestIdentity {
    pub(super) fn unique(prefix: &str) -> Self {
        let suffix = uuid::Uuid::now_v7().simple().to_string();
        Self {
            username: format!("{prefix}_{suffix}"),
            email: format!("{prefix}_{suffix}@example.com"),
            password: TEST_PASSWORD.to_string(),
        }
    }

    pub(super) fn registration_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "username": self.username,
            "password": self.password,
            "confirm_password": self.password,
            "email": self.email,
        })
    }

    pub(super) fn login_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "username": self.username,
            "password": self.password,
        })
    }

    pub(super) fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = password.into();
        self
    }
}

pub(super) struct TestSession {
    pub identity: TestIdentity,
    pub access_token: String,
    pub member: MemberProfile,
}

impl TestSession {
    pub(super) fn authorization(&self) -> Header<'static> {
        authorization(&self.access_token)
    }
}

#[derive(Clone, Copy)]
enum RegistrationKind {
    User,
    Administrator,
}

pub(super) fn client() -> Client {
    Client::tracked(crate::app::build()).expect("valid Rocket instance")
}

pub(super) fn json_body<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("test JSON should serialize")
}

pub(super) fn register_user(client: &Client, prefix: &str) -> TestSession {
    register(client, TestIdentity::unique(prefix), RegistrationKind::User)
}

pub(super) fn register_admin(client: &Client, prefix: &str) -> TestSession {
    register(
        client,
        TestIdentity::unique(prefix),
        RegistrationKind::Administrator,
    )
}

pub(super) fn register_identity(client: &Client, identity: TestIdentity) -> TestSession {
    register(client, identity, RegistrationKind::User)
}

pub(super) fn login(client: &Client, identity: &TestIdentity) -> TestSession {
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(json_body(&identity.login_payload()))
        .dispatch();

    authenticated_response(response, Status::Ok, identity.clone(), "login")
}

pub(super) fn authorization(token: impl AsRef<str>) -> Header<'static> {
    Header::new("Authorization", token.as_ref().to_owned())
}

pub(super) fn expect_error(
    response: LocalResponse<'_>,
    expected_status: Status,
    expected_code: &str,
) -> TestApiError {
    assert_eq!(response.status(), expected_status);
    let error = response
        .into_json::<TestApiError>()
        .expect("error response should be JSON");
    assert_eq!(error.code, expected_code);
    error
}

fn register(client: &Client, identity: TestIdentity, kind: RegistrationKind) -> TestSession {
    let mut request = client
        .post("/register")
        .header(ContentType::JSON)
        .body(json_body(&identity.registration_payload()));

    if matches!(kind, RegistrationKind::Administrator) {
        let administration_key = env::var(ADMINISTRATION_ENV)
            .expect("ADMINISTRATION_KEY must be configured for administrator tests");
        request = request.header(Header::new(ADMINISTRATION_HEADER, administration_key));
    }

    authenticated_response(
        request.dispatch(),
        Status::Created,
        identity,
        "registration",
    )
}

fn authenticated_response(
    response: LocalResponse<'_>,
    expected_status: Status,
    identity: TestIdentity,
    operation: &str,
) -> TestSession {
    assert_eq!(
        response.status(),
        expected_status,
        "{operation} should succeed"
    );
    let access_token = response
        .headers()
        .get_one("Authorization")
        .unwrap_or_else(|| panic!("{operation} should return an access token"))
        .to_owned();
    let member = response
        .into_json::<MemberProfile>()
        .unwrap_or_else(|| panic!("{operation} should return a member profile"));

    TestSession {
        identity,
        access_token,
        member,
    }
}
