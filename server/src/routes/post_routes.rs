use rocket::form::{Form, Lenient, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::State;

use crate::db::{DbConn, DbPool, get_db_connection};
use crate::models::post::{NewPost, Post, PostDetail};
use crate::routes::CustomResponse;
use crate::models::authentication::JWT;
use crate::service::{CrudOps, post_service::PostService};

fn post_authorization(conn: &mut DbConn, post_id: i32, jwt: JWT)
    -> Result<bool, status::Custom<String>> {

    if !jwt.claims.is_admin && !PostService::has_authorization(conn, post_id, jwt.claims.member_id){
        return Err(
            status::Custom(
                Status::Unauthorized,
                String::from("Only server members can access post details")
            )
        )
    }
    Ok(true)
}

#[post("/", data = "<form>")]
pub fn create(
    _token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<NewPost>>,
) -> CustomResponse<Post> {
    let mut conn = get_db_connection(pool)?;

    let input = form.into_inner().into_inner();
    match PostService::create(&mut conn, input){
        Ok(post) => Ok(
            status::Custom(
                Status::Ok,
                post
            )),
        Err(e) =>
            return Err::<status::Custom<Post>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Post creation error: {}", e)
                ))
    }
}

#[get("/<post_id>")]
pub fn get(
    token: JWT,
    pool: &State<DbPool>,
    post_id: i32
) -> CustomResponse<PostDetail>{
    let mut conn = get_db_connection(pool)?;
    post_authorization(&mut conn, post_id, token)?;

    match PostService::read_detail(&mut conn, post_id){
        Ok(post) => Ok(
            status::Custom(
                Status::Ok,
                post
            )),
        Err(e) =>
            return Err::<status::Custom<PostDetail>, status::Custom<String>>(
                status::Custom(
                    Status::NotFound,
                    format!("Post lookup error: {}", e)
                ))
    }
}


#[put("/<post_id>", data = "<form>")]
pub fn update(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Lenient<NewPost>>,
    post_id: i32
) -> CustomResponse<Post> {
    let mut conn = get_db_connection(pool)?;

    let post = PostService::read(&mut conn, post_id).map_err(|_| {
        status::Custom(Status::InternalServerError, String::from("Database connection error"))
    })?;
    if !token.claims.member_id == post.author_id() {
        return Err::<status::Custom<Post>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only post author can update contents.")))
    }

    let input = form.into_inner().into_inner();
    match PostService::update(&mut conn, post_id, input){
        Ok(post) => Ok(
            status::Custom(
                Status::Ok,
                post
            )),
        Err(e) =>
            return Err::<status::Custom<Post>, status::Custom<String>>(
                status::Custom(
                    Status::FailedDependency,
                    format!("Database update error: {}", e)
                ))
    }
}

#[delete("/<post_id>")]
pub fn delete(
    token: JWT,
    pool: &State<DbPool>,
    post_id: i32
) -> CustomResponse<String>{
    let mut conn = get_db_connection(pool)?;

    let post = PostService::read(&mut conn, post_id).map_err(|_| {
        status::Custom(Status::InternalServerError, String::from("Database connection error"))
    })?;
    if !token.claims.member_id == post.author_id() {
        return Err::<status::Custom<String>, status::Custom<String>>(status::Custom(Status::Unauthorized, String::from("Only post author can delete contents.")))
    }

    match PostService::delete(&mut conn, post_id){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, format!("Post number {} has been deleted.", post_id)))
        },
        Err(e) => {
            return Err::<status::Custom<String>, status::Custom<String>>(
                status::Custom(
                    Status::FailedDependency,
                    format!("Database deletion error: {}", e)
                ))
        }
    }
}