use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::ApiResponse;
use shared::models::server::channel::channel::{
    ChannelChangesetInput, ChannelDetailed, ChannelId, MultiChannel, MultiChannelPreview,
    NewChannelInput,
};
use shared::models::server::server::ServerId;

use super::service::ChannelService;

#[post("/<server_id>/channels", data = "<input>")]
fn create(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    input: Json<NewChannelInput>,
) -> ApiResponse<Json<ChannelDetailed>> {
    let channel =
        channel_service.create_channel(user.member_id(), server_id, input.into_inner())?;
    Ok(status::Custom(Status::Created, Json(channel)))
}

#[get("/<server_id>/channels/<channel_id>", rank = 2)]
fn get(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    channel_id: ChannelId,
) -> ApiResponse<Json<ChannelDetailed>> {
    let channel = channel_service.get_channel(user.member_id(), server_id, channel_id)?;
    Ok(status::Custom(Status::Ok, Json(channel)))
}

#[get("/<server_id>/channels")]
fn get_server_channels(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
) -> ApiResponse<Json<MultiChannel>> {
    let channels = channel_service.get_server_channels(user.member_id(), server_id)?;
    Ok(status::Custom(Status::Ok, Json(channels)))
}

#[get("/<server_id>/channels/preview")]
fn get_server_channel_previews(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
) -> ApiResponse<Json<MultiChannelPreview>> {
    let channels = channel_service.get_server_channel_previews(user.member_id(), server_id)?;
    Ok(status::Custom(Status::Ok, Json(channels)))
}

#[patch("/<server_id>/channels/<channel_id>", data = "<input>")]
fn update(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    channel_id: ChannelId,
    input: Json<ChannelChangesetInput>,
) -> ApiResponse<Json<ChannelDetailed>> {
    let channel = channel_service.update_channel(
        user.member_id(),
        server_id,
        channel_id,
        input.into_inner(),
    )?;
    Ok(status::Custom(Status::Ok, Json(channel)))
}

#[delete("/<server_id>/channels/<channel_id>")]
fn delete(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    channel_id: ChannelId,
) -> ApiResponse<()> {
    channel_service.delete_channel(user.member_id(), server_id, channel_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}

pub(in crate::server) fn routes() -> Vec<rocket::Route> {
    routes![
        create,
        get,
        get_server_channels,
        get_server_channel_previews,
        update,
        delete,
    ]
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::routes;

    #[test]
    fn channel_routes_use_plural_resource_paths() {
        let routes = routes();
        let signatures = routes
            .iter()
            .map(|route| (route.method, route.uri.to_string()))
            .collect::<Vec<_>>();

        assert!(routes.iter().any(|route| {
            route.uri.to_string() == "/<server_id>/channels/<channel_id>" && route.rank == 2
        }));

        assert!(signatures.contains(&(Method::Post, "/<server_id>/channels".into())));
        assert!(signatures.contains(&(Method::Patch, "/<server_id>/channels/<channel_id>".into())));
        assert!(signatures.contains(&(Method::Delete, "/<server_id>/channels/<channel_id>".into())));
        assert_eq!(signatures.len(), 6);
    }
}
