use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;

use crate::api_error::ApiError;
use crate::auth::guards::SystemAdmin;
use crate::db::{Db, DbPool};
use crate::member::models::MemberId;

use super::models::{RoleIdentifier, SystemRole, SystemRoleId};
use super::service::SystemRoleService;

#[derive(Deserialize)]
pub(super) struct RoleInput {
    name: String,
    authority_level: i32,
}

#[post("/", format = "json", data = "<input>")]
pub(super) fn create_role(
    admin: SystemAdmin,
    pool: &State<DbPool>,
    input: Json<RoleInput>,
) -> Result<status::Custom<Json<SystemRole>>, ApiError> {
    let mut conn = Db::connect(pool)?;
    let role = SystemRoleService::create_role(&mut conn, &input.name, input.authority_level)?;
    Ok(status::Custom(Status::Created, Json(role)))
}

#[get("/")]
pub(super) fn list_roles(
    _admin: SystemAdmin,
    pool: &State<DbPool>,
) -> Result<Json<Vec<SystemRole>>, ApiError> {
    let mut conn = Db::connect(pool)?;
    Ok(Json(SystemRoleService::get_roles(&mut conn)?))
}

#[put("/<role_id>", format = "json", data = "<input>")]
pub(super) fn update_role(
    admin: SystemAdmin,
    pool: &State<DbPool>,
    role_id: SystemRoleId,
    input: Json<RoleInput>,
) -> Result<Json<SystemRole>, ApiError> {
    let mut conn = Db::connect(pool)?;
    Ok(Json(SystemRoleService::update_role(
        &mut conn,
        role_id,
        &input.name,
        input.authority_level,
    )?))
}

#[delete("/<role_id>")]
pub(super) fn delete_role(
    admin: SystemAdmin,
    pool: &State<DbPool>,
    role_id: SystemRoleId,
) -> Result<status::NoContent, ApiError> {
    let mut conn = Db::connect(pool)?;
    SystemRoleService::delete_role(&mut conn, admin.member_id(), role_id)?;
    Ok(status::NoContent)
}

#[put("/<role_id>/members/<member_id>")]
pub(super) fn assign_role(
    admin: SystemAdmin,
    pool: &State<DbPool>,
    role_id: SystemRoleId,
    member_id: MemberId,
) -> Result<status::NoContent, ApiError> {
    let mut conn = Db::connect(pool)?;
    SystemRoleService::assign_role(&mut conn, admin.member_id(), member_id, role_id)?;
    Ok(status::NoContent)
}

#[get("/members/<member_id>")]
pub(super) fn assigned_role(
    _admin: SystemAdmin,
    pool: &State<DbPool>,
    member_id: MemberId,
) -> Result<Json<SystemRole>, ApiError> {
    let mut conn = Db::connect(pool)?;
    Ok(Json(SystemRoleService::get_member_role(
        &mut conn, member_id,
    )?))
}

#[delete("/members/<member_id>")]
pub(super) fn reset_role(
    admin: SystemAdmin,
    pool: &State<DbPool>,
    member_id: MemberId,
) -> Result<status::NoContent, ApiError> {
    let mut conn = Db::connect(pool)?;
    SystemRoleService::reset_role(&mut conn, admin.member_id(), member_id)?;
    Ok(status::NoContent)
}
