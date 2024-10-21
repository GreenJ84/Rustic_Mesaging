use crate::db::DbConn;

pub(crate) mod member_service;
pub(crate) mod server_service;
pub(crate) mod channel_service;
pub(crate) mod message_service;
pub(crate) mod post_service;
pub(crate) mod server_membership_service;
pub(crate) mod chat_service;
pub(crate) mod member_chat_service;
pub(crate) mod friend_service;
pub(crate) mod friend_request_service;
pub(crate) mod report_service;

pub trait CrudOps<N, T> {
    fn create(conn: &mut DbConn, entity: N) -> Result<T, diesel::result::Error>;
    fn read(conn: &mut DbConn, id: i32) -> Result<T, diesel::result::Error>;
    fn update(conn: &mut DbConn, id: i32, entity: N) -> Result<T, diesel::result::Error>;
    fn delete(conn: &mut DbConn, id: i32) -> Result<usize, diesel::result::Error>;
}