use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::ApiResponse;
use shared::models::member::MultiMemberPreview;
use shared::models::server::server::{
    MultiServer, MultiServerPreview, NewServerInput, ServerChangesetInput, ServerDetailed, ServerId,
};

use super::service::ServerService;

#[post("/", data = "<input>")]
fn create(
    user: SystemUser,
    server_service: &State<ServerService>,
    input: Json<NewServerInput>,
) -> ApiResponse<Json<ServerDetailed>> {
    let server = server_service.create_server(user.member_id(), input.into_inner())?;
    Ok(status::Custom(Status::Created, Json(server)))
}

#[get("/<server_id>")]
fn get(
    _user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<Json<ServerDetailed>> {
    let server = server_service.get_server(server_id)?;
    Ok(status::Custom(Status::Ok, Json(server)))
}

#[get("/all?<page>")]
fn get_servers_list(
    _user: SystemUser,
    server_service: &State<ServerService>,
    page: Option<u32>,
) -> ApiResponse<Json<MultiServer>> {
    let servers = server_service.get_servers_list(page.unwrap_or(0))?;
    Ok(status::Custom(Status::Ok, Json(servers)))
}

#[get("/search?<search_term>")]
fn search_servers(
    _user: SystemUser,
    server_service: &State<ServerService>,
    search_term: &str,
) -> ApiResponse<Json<MultiServerPreview>> {
    let servers = server_service.search_servers(search_term.to_string())?;
    Ok(status::Custom(Status::Ok, Json(servers)))
}

#[patch("/<server_id>", data = "<input>")]
fn update(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
    input: Json<ServerChangesetInput>,
) -> ApiResponse<Json<ServerDetailed>> {
    let server = server_service.update_server(server_id, user.member_id(), input.into_inner())?;
    Ok(status::Custom(Status::Ok, Json(server)))
}

#[delete("/<server_id>")]
fn delete(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<()> {
    server_service.delete_server(server_id, user.member_id())?;
    Ok(status::Custom(Status::NoContent, ()))
}

#[post("/<server_id>/members")]
fn join_server(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<()> {
    server_service.join_server(server_id, user.member_id())?;
    Ok(status::Custom(Status::Created, ()))
}

#[get("/<server_id>/members?<page>")]
fn get_server_members(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
    page: Option<u32>,
) -> ApiResponse<Json<MultiMemberPreview>> {
    let members =
        server_service.get_server_members(server_id, user.member_id(), page.unwrap_or(0))?;
    Ok(status::Custom(Status::Ok, Json(members)))
}

#[get("/servers?<page>")]
fn get_member_servers(
    user: SystemUser,
    server_service: &State<ServerService>,
    page: Option<u32>,
) -> ApiResponse<Json<MultiServerPreview>> {
    let servers = server_service.get_member_servers(user.member_id(), page.unwrap_or(0))?;
    Ok(status::Custom(Status::Ok, Json(servers)))
}

#[delete("/<server_id>/members/me")]
fn leave_server(
    user: SystemUser,
    server_service: &State<ServerService>,
    server_id: ServerId,
) -> ApiResponse<()> {
    server_service.leave_server(server_id, user.member_id())?;
    Ok(status::Custom(Status::NoContent, ()))
}

pub(super) fn routes() -> Vec<rocket::Route> {
    routes![
        create,
        get,
        get_servers_list,
        search_servers,
        update,
        delete,
        join_server,
        get_server_members,
        leave_server,
    ]
}

pub(crate) fn member_routes() -> Vec<rocket::Route> {
    routes![get_member_servers]
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::{member_routes, routes};

    #[test]
    fn server_routes_use_resource_oriented_methods_and_paths() {
        let signatures = routes()
            .into_iter()
            .map(|route| (route.method, route.uri.to_string()))
            .collect::<Vec<_>>();

        assert!(signatures.contains(&(Method::Post, "/".into())));
        assert!(signatures.contains(&(Method::Patch, "/<server_id>".into())));
        assert!(signatures.contains(&(Method::Post, "/<server_id>/members".into())));
        assert!(signatures.contains(&(Method::Delete, "/<server_id>/members/me".into())));
        assert_eq!(signatures.len(), 9);
    }

    #[test]
    fn member_route_owns_the_member_server_collection() {
        let routes = member_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, Method::Get);
        assert_eq!(routes[0].uri.to_string(), "/servers?<page>");
    }
}
