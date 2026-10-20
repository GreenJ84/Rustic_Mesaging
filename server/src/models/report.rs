use crate::member::models::{MemberProfile, MemberShort};
use chrono::NaiveDateTime;
use diesel::sql_types::{Text, Timestamp};
use diesel::{Queryable, QueryableByName};
use rocket::http::{ContentType, Status};
use rocket::response::{content, Responder};
use rocket::{Request, Response};
use serde::Serialize;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;
use std::path::Path;

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
    pub(crate) member: MemberProfile,
    #[sql_type = "Timestamp"]
    pub(crate) timestamp: NaiveDateTime,
}

#[derive(Serialize)]
pub struct ServerMemberReportCsv {
    id: i32,
    username: String,
    email: String,
    avatar: Option<String>,
    created_at: NaiveDateTime,
    joined_at: NaiveDateTime,
}
impl ServerMemberReportCsv {
    pub fn from_report_item(item: ServerMemberReportItem) -> Self {
        Self {
            id: item.member.id,
            username: item.member.username,
            email: item.member.email,
            avatar: item.member.avatar,
            created_at: item.member.created_at,
            joined_at: item.timestamp,
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
            .header(rocket::http::Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", self.filename),
            ))
            .sized_body(self.content.len(), Cursor::new(self.content))
            .finalize();
        Ok(response)
    }
}
