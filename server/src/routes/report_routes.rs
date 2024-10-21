use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use crate::db::{DbConn, DbPool, get_db_connection};
use crate::models::authentication::JWT;
use crate::models::member::MemberSafe;
use crate::models::report::{ActivityReportItem, ServerMemberReportItem};
use crate::service::member_service::MemberService;
use crate::service::report_service::ReportService;

#[get("/member")]
pub(crate) fn member_activity_report(
    token: JWT,
    pool: &State<DbPool>
) -> Json<Vec<ActivityReportItem>> {
    let mut conn = get_db_connection(pool).expect("Database connection expected");;

    Json(ReportService::generate_activity_report(&mut conn, token.claims.member_id))
}

#[get("/server/<server_id>")]
pub(crate) fn server_membership_report(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> Json<Vec<ServerMemberReportItem>> {
    let mut conn = get_db_connection(pool).expect("Database connection expected");

    Json(ReportService::generate_membership_report(&mut conn, server_id))
}