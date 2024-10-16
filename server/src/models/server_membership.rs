use chrono::NaiveDateTime;
use diesel::{Associations, Insertable, Queryable, Selectable};
use rocket::Response;
use rocket::response::Responder;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use crate::schema::server_membership;
use crate::models::{server::Server, member::Member};

#[derive(Insertable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = server_membership)]
pub struct NewServerMembership {
    pub member_id: i32,
    pub server_id: i32,
}


#[derive(Queryable, Associations, Serialize, Deserialize, Debug)]
#[diesel(table_name = server_membership)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[belongs_to(Server)]
#[belongs_to(Member)]
pub struct ServerMembership {
    #[diesel(sql_type = Int4)]
    pub server_id: i32,
    #[diesel(sql_type = Int4)]
    pub member_id: i32,
    #[diesel(sql_type = Timestamp)]
    joined_at: NaiveDateTime,
}
impl<'r> Responder<'r, 'r> for ServerMembership {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(Json(self).respond_to(req)?)
            .ok()
    }
}