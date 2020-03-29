use chrono::{DateTime, Utc};

#[derive(Queryable, Debug)]
pub struct FeedItem {
    pub id: i64,
    pub feed_id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub author: Option<String>,
    pub guid: Option<String>,

    pub publication_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
