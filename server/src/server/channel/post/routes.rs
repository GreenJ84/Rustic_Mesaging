use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::auth::SystemUser;
use crate::ApiResponse;
use shared::models::server::channel::channel::ChannelId;
use shared::models::server::channel::post::{
    MultiPostDetailed, NewPostInput, PostChangesetInput, PostDetailed, PostId,
};
use shared::models::server::server::ServerId;

use super::service::PostService;

#[post("/<server_id>/channels/<channel_id>/posts", data = "<input>")]
fn create(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    input: Json<NewPostInput>,
) -> ApiResponse<Json<PostDetailed>> {
    let post =
        post_service.create_post(user.member_id(), server_id, channel_id, input.into_inner())?;
    Ok(status::Custom(Status::Created, Json(post)))
}

#[get("/<server_id>/channels/<channel_id>/posts/<post_id>")]
fn get(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    post_id: PostId,
) -> ApiResponse<Json<PostDetailed>> {
    let post = post_service.get_post_detailed(user.member_id(), server_id, channel_id, post_id)?;
    Ok(status::Custom(Status::Ok, Json(post)))
}

#[get("/<server_id>/channels/<channel_id>/posts?<page>")]
fn get_page(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    page: Option<u32>,
) -> ApiResponse<Json<MultiPostDetailed>> {
    let posts =
        post_service.get_post_page(user.member_id(), server_id, channel_id, page.unwrap_or(0))?;
    Ok(status::Custom(Status::Ok, Json(posts)))
}

#[patch("/<server_id>/channels/<channel_id>/posts/<post_id>", data = "<input>")]
fn update(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    post_id: PostId,
    input: Json<PostChangesetInput>,
) -> ApiResponse<Json<PostDetailed>> {
    let post = post_service.update_post(
        user.member_id(),
        server_id,
        channel_id,
        post_id,
        input.into_inner(),
    )?;
    Ok(status::Custom(Status::Ok, Json(post)))
}

#[delete("/<server_id>/channels/<channel_id>/posts/<post_id>")]
fn delete(
    user: SystemUser,
    post_service: &State<PostService>,
    server_id: ServerId,
    channel_id: ChannelId,
    post_id: PostId,
) -> ApiResponse<()> {
    post_service.delete_post(user.member_id(), server_id, channel_id, post_id)?;
    Ok(status::Custom(Status::NoContent, ()))
}

pub(in crate::server) fn routes() -> Vec<rocket::Route> {
    routes![create, get, get_page, update, delete]
}

#[cfg(test)]
mod tests {
    use rocket::http::Method;

    use super::routes;

    #[test]
    fn post_routes_use_nested_resource_paths_and_patch_updates() {
        let signatures = routes()
            .into_iter()
            .map(|route| (route.method, route.uri.to_string()))
            .collect::<Vec<_>>();

        assert!(signatures.contains(&(
            Method::Post,
            "/<server_id>/channels/<channel_id>/posts".into()
        )));
        assert!(signatures.contains(&(
            Method::Patch,
            "/<server_id>/channels/<channel_id>/posts/<post_id>".into()
        )));
        assert_eq!(signatures.len(), 5);
    }
}
