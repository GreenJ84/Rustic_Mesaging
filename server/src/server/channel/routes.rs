use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::server::models::ServerId;
use crate::ApiResponse;

use super::models::{
    ChannelChangesetInput, ChannelDetails, ChannelId, MultiChannelDetails, MultiChannelPreview,
    NewChannelInput,
};
use super::service::ChannelService;

#[post("/<server_id>/channel", data = "<form>")]
pub fn create(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    form: Form<Strict<NewChannelInput>>,
) -> ApiResponse<Json<ChannelDetails>> {
    let input = form.into_inner().into_inner();
    let new_channel = channel_service.create_channel(user.member_id(), server_id, input)?;
    Ok(status::Custom(Status::Created, Json(new_channel)))
}

#[get("/<server_id>/channel/<channel_id>")]
pub fn get(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    channel_id: ChannelId,
) -> ApiResponse<Json<ChannelDetails>> {
    let channel = channel_service.get_channel(user.member_id(), server_id, channel_id)?;
    Ok(status::Custom(Status::Ok, Json(channel)))
}

#[get("/<server_id>/channels")]
pub fn get_server_channels(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
) -> ApiResponse<Json<MultiChannelDetails>> {
    let channels = channel_service.get_server_channels(user.member_id(), server_id)?;
    Ok(status::Custom(Status::Ok, Json(channels)))
}

#[get("/<server_id>/channels/preview")]
pub fn get_server_channel_previews(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
) -> ApiResponse<Json<MultiChannelPreview>> {
    let channels = channel_service.get_server_channel_previews(user.member_id(), server_id)?;
    Ok(status::Custom(Status::Ok, Json(channels)))
}

#[patch("/<server_id>/channel/<channel_id>", data = "<form>")]
pub fn update(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    channel_id: ChannelId,
    form: Form<Strict<ChannelChangesetInput>>,
) -> ApiResponse<Json<ChannelDetails>> {
    let input = form.into_inner().into_inner();

    let channel = channel_service.update_channel(user.member_id(), server_id, channel_id, input)?;
    Ok(status::Custom(Status::Ok, Json(channel)))
}

#[delete("/<server_id>/channel/<channel_id>")]
pub fn delete(
    user: SystemUser,
    channel_service: &State<ChannelService>,
    server_id: ServerId,
    channel_id: ChannelId,
) -> ApiResponse<()> {
    channel_service.delete_channel(user.member_id(), server_id, channel_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}

// #[get("/<channel_id>/posts")]
// pub fn get_channel_posts(
//     user: SystemUser,
//     channel_service: &State<ChannelService>,
//     channel_id: ChannelId,
// ) -> ApiResponse<MultiPost> {
//     channel_authorized(&mut conn, channel_id, token)?;

//     match channel_service.get_channel_posts(&mut conn, channel_id) {
//         Ok(posts) => Ok(status::Custom(Status::Ok, posts)),
//         Err(e) => {
//             return Err::<status::Custom<MultiPost>, status::Custom<String>>(status::Custom(
//                 Status::NotFound,
//                 format!("Channel post lookup error: {}", e),
//             ))
//         }
//     }
// }
