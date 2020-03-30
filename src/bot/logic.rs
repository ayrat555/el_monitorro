use crate::db::telegram;
use crate::models::telegram_chat::TelegramChat;
use crate::models::telegram_subscription::TelegramSubscription;
use url::{ParseError, Url};

pub enum SubscriptionError {
    InvalidRssUrl,
    SubscriptionAlreadyExists,
    SubscriptionCountLimit,
}

pub fn create_subscription(
    chat_id: i64,
    feed_id: i64,
    rss_url: &str,
) -> Result<TelegramSubscription, SubscriptionError> {
    validate_rss_url(rss_url)?;

    Err(SubscriptionError::SubscriptionAlreadyExists)
}

fn validate_rss_url(rss_url: &str) -> Result<(), SubscriptionError> {
    match Url::parse(rss_url) {
        Ok(_) => Ok(()),
        ParseError => Err(SubscriptionError::InvalidRssUrl),
    }
}
