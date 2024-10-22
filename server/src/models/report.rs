use chrono::NaiveDateTime;
use diesel::{Queryable, QueryableByName};
use serde::Serialize;
use crate::models::member::{MemberSafe, MemberShort};
use diesel::sql_types::{Text, Timestamp};
use rocket::{Request, Response};
use rocket::response::{Responder, content};
use rocket::http::{ContentType, Status};
use std::io::Cursor;
use std::fs::File;
use std::path::Path;
use std::io::Write;

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
    member: MemberSafe,
    #[sql_type = "Timestamp"]
    timestamp: NaiveDateTime,
}

#[derive(Serialize)]
pub struct ServerMemberReportCsv {
    id: i32,
    username: String,
    email: String,
    avatar: Option<String>,
    is_admin: bool,
    created_at: NaiveDateTime,
    joined_at: NaiveDateTime,
}
impl ServerMemberReportCsv{
    pub fn from_report_item(item: ServerMemberReportItem) -> Self{
        Self {
            id: item.member.id(),
            username: item.member.username().to_string(),
            email: item.member.email().to_string(),
            avatar: item.member.avatar().clone(),
            is_admin: item.member.is_admin(),
            created_at: item.member.created_at(),
            joined_at: Default::default(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct CsvDownload {
    filename: String,
    content: String,
}

impl CsvDownload {
    pub fn new<T: Serialize>(data: &[T], filename: &str) -> Result<Self, String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        log::info!("Got writer");
        for record in data {
            wtr.serialize(record).map_err(|e| e.to_string())?;
        }
        log::info!("Wrote data");
        let csv_string = String::from_utf8(wtr.into_inner().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        log::info!("turned string");
        Ok(CsvDownload {
            filename: filename.to_string(),
            content: csv_string,
        })
    }
}

impl<'r> Responder<'r, 'r> for CsvDownload {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'r> {
        let content_type = ContentType::new("text", "csv");
        let mut response = Response::build()
            .header(content_type)
            .header(rocket::http::Header::new("Content-Disposition", format!("attachment; filename=\"{}\"", self.filename)))
            .sized_body(self.content.len(), Cursor::new(self.content))
            .finalize();
        Ok(response)
    }
}