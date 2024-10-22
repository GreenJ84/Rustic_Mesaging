use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rocket::response;

use crate::db::{DbConn, DbPool, get_db_connection};
use crate::models::authentication::JWT;
use crate::models::member::MemberSafe;
use crate::models::report::{ActivityReportItem, CsvDownload, ServerMemberReportCsv, ServerMemberReportItem};
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

#[get("/member/csv")]
pub(crate)  fn download_member_activity_report(
    token: JWT,
    pool: &State<DbPool>
) -> Result<CsvDownload, Status> {
    let mut conn = get_db_connection(pool).expect("Database connection expected");

    let data = ReportService::generate_activity_report(&mut conn, token.claims.member_id);

    CsvDownload::new(&data, "activity_report.csv").map_err(|_| Status::InternalServerError)
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

#[get("/server/<server_id>/csv")]
pub(crate) fn download_server_membership_report(
    token: JWT,
    pool: &State<DbPool>,
    server_id: i32
) -> Result<CsvDownload, Status> {
    let mut conn = get_db_connection(pool).expect("Database connection expected");

    let data = ReportService::generate_membership_report(&mut conn, server_id);
    let csv_data: Vec<ServerMemberReportCsv> = data.into_iter().map(|item| ServerMemberReportCsv::from_report_item(item)).collect();
    CsvDownload::new(&csv_data, &format!("server_{}.csv", server_id)).map_err(|_| Status::InternalServerError)
}