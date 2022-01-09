use chrono::{DateTime, Utc};

#[derive(Queryable, Debug, Clone)]
pub struct FeedItem {
    pub feed_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub link: String,
    pub author: Option<String>,
    pub guid: Option<String>,

    pub publication_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub content_hash: String,
}
