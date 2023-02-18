use crate::schema::telegram_subscriptions;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = telegram_subscriptions)]
#[diesel(primary_key(chat_id, feed_id))]
pub struct TelegramSubscription {
    pub chat_id: i64,
    pub feed_id: i64,

    pub last_delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub template: Option<String>,
    pub filter_words: Option<Vec<String>>,
    pub has_updates: bool,
    pub external_id: Uuid,
    pub thread_id: Option<i32>,
}
