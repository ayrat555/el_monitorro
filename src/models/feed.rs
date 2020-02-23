use chrono::{DateTime, Utc};

#[derive(Queryable, Debug)]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub link: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
