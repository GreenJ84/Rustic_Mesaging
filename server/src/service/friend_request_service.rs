use diesel::result::{Error};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl, ExpressionMethods};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::json::Json;

use crate::db::DbConn;
use crate::models::friend::{Friend, NewFriend};
use crate::models::friend_request::{NewFriendRequest, FriendRequest};
use crate::schema::friend_request::dsl;
use crate::service::friend_service::FriendService;

pub struct FriendRequestService;

impl FriendRequestService {
    pub fn create_request(conn: &mut DbConn, new_request: NewFriendRequest) -> Result<RequestResolve, Error> {
        match diesel::insert_into(dsl::friend_request)
            .values(&new_request)
            .get_result::<FriendRequest>(conn)
        {
            Ok(friend_request) => Ok(RequestResolve::NotResolved(friend_request)),  // Successfully inserted friend request
            Err(Error::NotFound) => {
                let new_friend = NewFriend::new(
                    new_request.sender_id(),
                    new_request.receiver_id()
                );
                let friend = FriendService::read(
                    conn,
                    new_friend.member_id(),
                    new_friend.friend_id()
                )?;
                Ok(RequestResolve::Resolved(friend))
            }, // Trigger returned NULL, friendship was resolved
            Err(e) => Err(e),
        }
    }

    pub fn is_requested(
        conn: &mut DbConn, sender_id: i32, receiver_id: i32
    ) -> bool {
        Self::get_request(conn, sender_id, receiver_id).is_ok()
    }
    pub fn get_request(
        conn: &mut DbConn, sender_id: i32, receiver_id: i32
    ) -> Result<FriendRequest, Error>{
        let friend = dsl::friend_request.find((sender_id, receiver_id))
            .first::<FriendRequest>(conn)
            .optional()?;
        match friend {
            Some(friend) => Ok(friend),
            None => { Err(Error::NotFound) }
        }
    }

    pub fn update_request(
        conn: &mut DbConn, update: NewFriendRequest
    ) -> Result<FriendRequest, Error>{
        diesel::update(dsl::friend_request.find(
            (update.sender_id(), update.receiver_id())
        ))
            .set(dsl::note.eq(update.note().to_owned()))
            .get_result::<FriendRequest>(conn)
    }

    fn delete_request(conn: &mut DbConn, sender_id: i32, receiver_id: i32) -> Result<usize, Error> {
        diesel::delete(dsl::friend_request.find((sender_id, receiver_id))).execute(conn)
    }
    pub fn accept_request(
        conn: &mut DbConn, sender_id: i32, receiver_id: i32
    ) -> Result<Friend, Error>{
        let friend = FriendService::add_friend(conn, NewFriend::new(sender_id, receiver_id))?;
        Self::delete_request(conn, sender_id, receiver_id)?;
        Ok(friend)
    }
    pub fn deny_request(
        conn: &mut DbConn, sender_id: i32, receiver_id: i32
    ) -> Result<usize, Error> {
        Self::delete_request(conn, sender_id, receiver_id)
    }
}

pub enum RequestResolve {
    NotResolved(FriendRequest),
    Resolved(Friend)
}
impl<'r> Responder<'r, 'r> for RequestResolve {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        match self {
            RequestResolve::NotResolved(request) => {
                Response::build_from(Json(request).respond_to(req)?)
                    .ok()
            },
            RequestResolve::Resolved(friend) => {
                Response::build_from(Json(friend).respond_to(req)?)
                    .ok()
            }
        }
    }
}