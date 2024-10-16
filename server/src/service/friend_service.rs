use diesel::result::{Error};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use crate::db::DbConn;
use crate::models::friend::{NewFriend, Friend};
use crate::schema::friend::dsl;

pub struct FriendService;

impl FriendService {
    // Create
    pub fn add_friend(conn: &mut DbConn, new_friendship: NewFriend) -> Result<Friend, Error> {
        diesel::insert_into(dsl::friend)
            .values(&new_friendship)
            .get_result::<Friend>(conn)
    }
    // Get
    pub fn is_friend(
        conn: &mut DbConn, member_id: i32, friend_id: i32
    ) -> bool {
        Self::read(conn, member_id, friend_id).is_ok()
    }
    pub fn read(
        conn: &mut DbConn, member_id: i32, friend_id: i32
    ) -> Result<Friend, Error> {
        let friend = dsl::friend.find((member_id, friend_id))
            .first::<Friend>(conn)
            .optional()?;

        match friend {
            Some(friend) => Ok(friend),
            None => Err(Error::NotFound)
        }
    }
    // Delete
    pub fn remove_friend(conn: &mut DbConn, member_id: i32, friend_id: i32) -> Result<usize, Error> {
        diesel::delete(dsl::friend.find((member_id, friend_id))).execute(conn)
    }
}