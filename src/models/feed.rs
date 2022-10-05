use crate::schema::feeds;
use chrono::{DateTime, Utc};

#[derive(Queryable, Identifiable, Debug, Eq, PartialEq)]
#[diesel(table_name = feeds)]
pub struct Feed {
    pub id: i64,
    pub title: Option<String>,
    pub link: String,
    pub error: Option<String>,
    pub description: Option<String>,

    pub synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub feed_type: String,

    pub sync_retries: i32,
    pub sync_skips: i32,

    pub content_fields: Option<Vec<String>>,
}
