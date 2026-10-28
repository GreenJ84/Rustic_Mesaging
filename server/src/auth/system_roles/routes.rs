use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;

use crate::api::ApiError;
use crate::auth::guards::SystemAdmin;
use crate::member::models::MemberId;

use super::models::{SystemRole, SystemRoleId};
use crate::auth::service::AuthService;

#[derive(Deserialize)]
pub(super) struct RoleInput {
    name: String,
    authority_level: i32,
}

#[post("/", format = "json", data = "<input>")]
pub(super) fn create_role(
    _admin: SystemAdmin,
    auth_service: &State<AuthService>,
    input: Json<RoleInput>,
) -> Result<status::Custom<Json<SystemRole>>, ApiError> {
    let role = auth_service.create_role(&input.name, input.authority_level)?;
    Ok(status::Custom(Status::Created, Json(role)))
}

#[get("/")]
pub(super) fn list_roles(
    _admin: SystemAdmin,
    auth_service: &State<AuthService>,
) -> Result<Json<Vec<SystemRole>>, ApiError> {
    Ok(Json(auth_service.get_roles()?))
}

#[put("/<role_id>", format = "json", data = "<input>")]
pub(super) fn update_role(
    _admin: SystemAdmin,
    auth_service: &State<AuthService>,
    role_id: SystemRoleId,
    input: Json<RoleInput>,
) -> Result<Json<SystemRole>, ApiError> {
    Ok(Json(auth_service.update_role(
        role_id,
        &input.name,
        input.authority_level,
    )?))
}

#[delete("/<role_id>")]
pub(super) fn delete_role(
    admin: SystemAdmin,
    auth_service: &State<AuthService>,
    role_id: SystemRoleId,
) -> Result<status::NoContent, ApiError> {
    auth_service.delete_role(admin.member_id(), role_id)?;
    Ok(status::NoContent)
}

#[put("/<role_id>/members/<member_id>")]
pub(super) fn assign_role(
    admin: SystemAdmin,
    auth_service: &State<AuthService>,
    role_id: SystemRoleId,
    member_id: MemberId,
) -> Result<status::NoContent, ApiError> {
    auth_service.assign_role(admin.member_id(), member_id, role_id)?;
    Ok(status::NoContent)
}

#[get("/members/<member_id>")]
pub(super) fn assigned_role(
    _admin: SystemAdmin,
    auth_service: &State<AuthService>,
    member_id: MemberId,
) -> Result<Json<SystemRole>, ApiError> {
    Ok(Json(auth_service.get_member_role(member_id)?))
}

#[delete("/members/<member_id>")]
pub(super) fn reset_role(
    admin: SystemAdmin,
    auth_service: &State<AuthService>,
    member_id: MemberId,
) -> Result<status::NoContent, ApiError> {
    auth_service.reset_role(admin.member_id(), member_id)?;
    Ok(status::NoContent)
}
