use chrono::NaiveDateTime;
use diesel::{Queryable, QueryableByName};
use serde::Serialize;
use crate::models::member::{MemberSafe, MemberShort};
use diesel::sql_types::{Text, Timestamp};

#[derive(Debug, QueryableByName, Serialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ActivityReportItem {
    #[sql_type = "Text"]
    pub(crate) entity: String,

    #[sql_type = "Text"]
    pub(crate) description: String,

    #[sql_type = "Timestamp"]
    pub(crate) timestamp: NaiveDateTime,
}

#[derive(Debug, QueryableByName, Serialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ServerMemberReportItem {
    #[diesel(embed)]
    user: MemberSafe,
    #[sql_type = "Timestamp"]
    timestamp: NaiveDateTime,
}