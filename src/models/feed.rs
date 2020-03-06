use crate::schema::feeds;
use chrono::{DateTime, Utc};

#[derive(Queryable, Identifiable, Debug, Eq, PartialEq)]
#[table_name = "feeds"]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub link: String,
    pub error: Option<String>,
    pub description: String,

    pub synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
