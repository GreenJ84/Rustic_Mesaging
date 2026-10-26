use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::server::channel::models::ChannelId;
use crate::server::channel::post::models::{MultiPost, PostId};
use crate::server::channel::post::service::PostService;
use crate::server::models::ServerId;
use crate::ApiResponse;

use super::models::{NewPostInput, Post, PostChangesetInput, PostDetail};

#[post("/<server_id>/channels/<channel_id>/posts", data = "<form>")]
pub fn create(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    form: Form<Strict<NewPostInput>>,
) -> ApiResponse<Json<Post>> {
    let input = form.into_inner().into_inner();
    let post = post_service.create_post(user.member_id(), server_id, channel_id, input)?;
    Ok(status::Custom(Status::Created, Json(post)))
}

#[get("/<server_id>/channels/<channel_id>/posts/<post_id>")]
pub fn get(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    post_id: PostId,
) -> ApiResponse<Json<PostDetail>> {
    let post = post_service.get_post_detail(user.member_id(), server_id, channel_id, post_id)?;
    Ok(status::Custom(Status::Ok, Json(post)))
}

#[get("/<server_id>/channels/<channel_id>/posts?<page>")]
pub fn get_page(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    page: Option<u32>,
) -> ApiResponse<Json<MultiPost>> {
    let page = page.unwrap_or(0);
    let page = post_service.get_post_page(user.member_id(), server_id, channel_id, page)?;
    Ok(status::Custom(Status::Ok, Json(page)))
}

#[put("/<server_id>/channels/<channel_id>/posts/<post_id>", data = "<form>")]
pub fn update(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    form: Form<Strict<PostChangesetInput>>,
    post_id: PostId,
) -> ApiResponse<Json<Post>> {
    let input = form.into_inner().into_inner();
    let post = post_service.update_post(user.member_id(), server_id, channel_id, post_id, input)?;
    Ok(status::Custom(Status::Ok, Json(post)))
}

#[delete("/<server_id>/channels/<channel_id>/posts/<post_id>")]
pub fn delete(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    post_id: PostId,
) -> ApiResponse<()> {
    post_service.delete_post(user.member_id(), server_id, channel_id, post_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}
