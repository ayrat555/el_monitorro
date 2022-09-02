use crate::schema::telegram_chats;
use chrono::{DateTime, Utc};

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = telegram_chats)]
#[diesel(primary_key(id))]
pub struct TelegramChat {
    pub id: i64,
    pub kind: String,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: Option<String>,
    pub utc_offset_minutes: Option<i32>,
    pub template: Option<String>,
    pub filter_words: Option<Vec<String>>,
}
