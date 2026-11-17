//! Direct integration tests for Member service policy and orchestration.

use super::models::MemberUpdateInput;
use super::service::MemberService;
use crate::api::ApiError;
use crate::tests::common::{client, register_user};
use rocket::http::Status;

#[test]
fn member_service_owns_reads_search_and_updates() {
    let client = client();
    let registered = register_user(&client, "member_service_lifecycle");
    let service = client.rocket().state::<MemberService>().unwrap();

    assert_eq!(
        service.get(registered.member.id).unwrap().id(),
        registered.member.id
    );
    assert_eq!(
        service.get_detailed(registered.member.id).unwrap().email,
        registered.member.email
    );
    assert_eq!(
        service
            .get_by_username(format!("  {}  ", registered.member.username))
            .unwrap()
            .id,
        registered.member.id
    );
    assert!(service
        .search(registered.member.username.clone())
        .unwrap()
        .members
        .iter()
        .any(|member| member.id == registered.member.id));

    let updated_username = format!("member_service_updated_{}", uuid::Uuid::now_v7().simple());
    let updated = service
        .update(
            registered.member.id,
            MemberUpdateInput {
                username: Some(format!("  {updated_username}  ")),
                email: None,
                avatar: Some("  robot  ".into()),
            },
        )
        .unwrap();
    assert_eq!(updated.username, updated_username);
    assert_eq!(updated.avatar, "robot");
}

#[test]
fn member_service_classifies_identity_conflicts_and_invalid_inputs() {
    let client = client();
    let existing = register_user(&client, "member_service_existing");
    let changing = register_user(&client, "member_service_changing");
    let service = client.rocket().state::<MemberService>().unwrap();

    for input in [
        MemberUpdateInput {
            username: Some(existing.member.username.clone()),
            email: None,
            avatar: None,
        },
        MemberUpdateInput {
            username: None,
            email: Some(existing.member.email.clone()),
            avatar: None,
        },
    ] {
        assert_eq!(
            ApiError::from(service.update(changing.member.id, input).unwrap_err()).status(),
            Status::Conflict
        );
    }

    for input in [
        MemberUpdateInput {
            username: None,
            email: None,
            avatar: None,
        },
        MemberUpdateInput {
            username: Some("ab".into()),
            email: None,
            avatar: None,
        },
        MemberUpdateInput {
            username: None,
            email: Some("not-an-email".into()),
            avatar: None,
        },
    ] {
        assert_eq!(
            ApiError::from(service.update(changing.member.id, input).unwrap_err()).status(),
            Status::BadRequest
        );
    }
}

#[test]
fn member_service_rejects_invalid_and_missing_lookups() {
    let client = client();
    let service = client.rocket().state::<MemberService>().unwrap();

    assert_eq!(
        ApiError::from(service.get_by_username("ab".into()).unwrap_err()).status(),
        Status::BadRequest
    );
    assert_eq!(
        ApiError::from(
            service
                .get_by_username(format!(
                    "missing_member_service_{}",
                    uuid::Uuid::now_v7().simple()
                ))
                .unwrap_err(),
        )
        .status(),
        Status::NotFound
    );
    assert_eq!(
        ApiError::from(service.search("   ".into()).unwrap_err()).status(),
        Status::BadRequest
    );
}
