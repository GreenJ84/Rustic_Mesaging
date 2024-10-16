use rocket::http::Status;
use rocket::response::status;
use rocket::State;
use rocket::serde::json::Json;
use crate::db::{DbPool, get_db_connection};
use crate::models::{
    friend::{NewFriend, Friend},
    authentication::JWT
};
use crate::routes::CustomResponse;
use crate::service::friend_service::FriendService;



#[post("/<friend_id>")]
pub fn add_friend(
    token: JWT,
    pool: &State<DbPool>,
    friend_id: i32
) -> CustomResponse<Friend> {
    let mut conn = get_db_connection(pool)?;

    let new_friendship = NewFriend::new(token.claims.member_id, friend_id);

    match FriendService::add_friend(&mut conn, new_friendship){
        Ok(friend) => Ok(status::Custom(
            Status::Created,
            friend
        )),
        Err(e) =>
            return Err::<status::Custom<Friend>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Friendship creation error: {}", e)
                ))
    }
}

#[get("/<friend_id>")]
pub fn get_friend_status(
    token: JWT,
    pool: &State<DbPool>,
    friend_id: i32
) -> CustomResponse<Json<(bool, String)>>{
    let mut conn = get_db_connection(pool)?;
    let new_friendship = NewFriend::new(token.claims.member_id, friend_id);


    let is_friend =  FriendService::is_friend(&mut conn, new_friendship.member_id(), new_friendship.friend_id());
    Ok(status::Custom(
        Status::Ok,
        Json((is_friend, format!("You are {}a friend with member {}", if is_friend { "" } else { "NOT " }, friend_id ))
    )))
}

#[put("/<_friend_id>")]
pub fn update_friend_status(
    _token: JWT,
    _friend_id: i32,
) -> CustomResponse<()> {
    Err(status::Custom(Status::NotImplemented, "No ability to update, just add or remove".to_string()))
}

#[delete("/<friend_id>")]
pub fn remove_friend(
    token: JWT,
    pool: &State<DbPool>,
    friend_id: i32
) -> CustomResponse<()>{
    let mut conn = get_db_connection(pool)?;
    let new_friendship = NewFriend::new(token.claims.member_id, friend_id);
    if !FriendService::is_friend(&mut conn, new_friendship.member_id(), new_friendship.friend_id()){
        return Err::<status::Custom<()>, status::Custom<String>>(
            status::Custom(
                Status::Conflict,
                format!("You are not currently a friend with Member ID: {}. Cannot remove.", friend_id)
            )
        )
    }

    match FriendService::remove_friend(&mut conn, new_friendship.member_id(), new_friendship.friend_id()){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, ()))
        },
        Err(e) => {
            return Err::<status::Custom<()>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Database deletion error: {}", e)
                ))
        }
    }
}