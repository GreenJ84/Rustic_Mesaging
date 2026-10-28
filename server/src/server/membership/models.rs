use chrono::NaiveDateTime;
use diesel::{Associations, Insertable, Queryable, Selectable};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::Response;

use crate::member::models::{Member, MemberId};
use crate::schema::server_membership;
use crate::server::models::{Server, ServerId};

#[derive(Insertable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = server_membership)]
pub struct NewServerMembership {
    pub member_id: MemberId,
    pub server_id: ServerId,
}

#[derive(Queryable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = server_membership)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Member))]
#[diesel(belongs_to(Server))]
pub struct ServerMembership {
    #[diesel(sql_type = Int4)]
    pub member_id: MemberId,
    #[diesel(sql_type = Int4)]
    pub server_id: ServerId,
    #[diesel(sql_type = Timestamp)]
    joined_at: NaiveDateTime,
}
