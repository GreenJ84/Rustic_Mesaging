use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::member::models::MultiMemberPreview;

use super::service::ServerService;
use crate::server::models::{
    MultiServerPreview, ServerChangesetInput, ServerCreateInput, ServerDetails, ServerId,
};
use crate::ApiResponse;

#[post("/", data = "<form>")]
pub fn create(
    user: SystemUser,
    server_service: &State<ServerService>,
    form: Form<Strict<ServerCreateInput>>,
) -> ApiResponse<Json<ServerDetails>> {
    let input = form.into_inner().into_inner();
    let server = server_service.create_server(user.member_id(), input)?;
    Ok(status::Custom(Status::Created, Json(server)))
}

#[get("/<server_id>")]
pub fn get(
    _user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<Json<ServerDetails>> {
    let server = server_service.get_server(server_id)?;
    Ok(status::Custom(Status::Ok, Json(server)))
}

#[get("/all?<page>")]
pub fn get_servers_list(
    _user: SystemUser,
    server_service: &State<ServerService>,
    page: Option<u32>,
) -> ApiResponse<Json<MultiServerPreview>> {
    let page = page.unwrap_or(0);
    let servers = server_service.get_servers_list(page)?;
    Ok(status::Custom(Status::Ok, Json(servers)))
}

#[get("/search?<search_term>")]
pub fn search_servers(
    _user: SystemUser,
    server_service: &State<ServerService>,
    search_term: &str,
) -> ApiResponse<Json<MultiServerPreview>> {
    let servers = server_service.search_servers(search_term.to_string())?;
    Ok(status::Custom(Status::Ok, Json(servers)))
}

#[patch("/<server_id>", data = "<form>")]
pub fn update(
    user: SystemUser,
    server_service: &State<ServerService>,
    form: Form<Strict<ServerChangesetInput>>,
    server_id: ServerId,
) -> ApiResponse<Json<ServerDetails>> {
    let input = form.into_inner().into_inner();
    let server = server_service.update_server(server_id, user.member_id(), input)?;
    Ok(status::Custom(Status::Ok, Json(server)))
}

#[delete("/<server_id>")]
pub fn delete(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<()> {
    server_service.delete_server(server_id, user.member_id())?;
    Ok(status::Custom(Status::NoContent, ()))
}

#[post("/<server_id>/join")]
pub fn join_server(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<()> {
    server_service.join_server(server_id, user.member_id())?;
    Ok(status::Custom(Status::Created, ()))
}

#[get("/<server_id>/members?<page>")]
pub fn get_server_members(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
    page: Option<u32>,
) -> ApiResponse<Json<MultiMemberPreview>> {
    let page = page.unwrap_or(0);
    let members = server_service.get_server_members(server_id, user.member_id(), page)?;
    Ok(status::Custom(Status::Ok, Json(members)))
}

#[get("/servers?<page>")]
pub fn get_member_servers(
    user: SystemUser,
    server_service: &State<ServerService>,
    page: Option<u32>,
) -> ApiResponse<Json<MultiServerPreview>> {
    let page = page.unwrap_or(0);
    let servers = server_service.get_member_servers(user.member_id(), page)?;
    Ok(status::Custom(Status::Ok, Json(servers)))
}

#[delete("/<server_id>/leave")]
pub fn leave_server(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<()> {
    server_service.leave_server(server_id, user.member_id())?;
    Ok(status::Custom(Status::NoContent, ()))
}

// #[get("/<server_id>/channels")]
// pub fn get_server_channels(
//     user: SystemUser,
//     server_service: &State<ServerService>,
//     server_id: ServerId,
// ) -> ApiResponse<MultiChannel> {
//     let mut conn = get_db_connection(pool)?;
//     if !MembershipService::has_membership(&mut conn, server_id, token.claims.member_id) {
//         return Err::<status::Custom<MultiChannel>, status::Custom<String>>(status::Custom(
//             Status::Unauthorized,
//             String::from("Only server members can view channels within"),
//         ));
//     }

//     match ServerService::get_server_channels(&mut conn, server_id) {
//         Ok(channels) => Ok(status::Custom(Status::Ok, MultiChannel { channels })),
//         Err(e) => {
//             return Err::<status::Custom<MultiChannel>, status::Custom<String>>(status::Custom(
//                 Status::NotFound,
//                 format!("Server channels lookup error: {}", e),
//             ))
//         }
//     }
// }
