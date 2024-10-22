use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::models::member::Member;

#[derive(Deserialize, Debug)]
pub(crate) struct ActivityReportItem {
    pub(crate) entity: String,
    pub(crate) description: String,
    pub(crate) timestamp: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ServerMemberReportItem {
    pub member: Member,
    pub(crate) timestamp: NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct CsvDownload {
    filename: String,
    pub(crate) content: String,
}