use chrono::{DateTime, Utc};

#[derive(Queryable, Debug)]
pub struct FeedItem {
    pub id: i32,
    pub feed_id: i32,
    pub title: String,
    pub description: String,
    pub link: String,
    pub author: String,
    pub guid: String,
    pub categories: Vec<String>,

    pub publication_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
