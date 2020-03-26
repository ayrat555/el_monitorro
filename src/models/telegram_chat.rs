use chrono::{DateTime, Utc};

#[derive(Queryable, Debug)]
pub struct TelegramChat {
    pub id: i32,
    pub kind: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
