use crate::bot::api;
use crate::db;
use crate::db::feeds;
use crate::db::telegram;
use crate::models::feed::Feed;
use crate::models::feed_item::FeedItem;
use crate::models::telegram_chat::TelegramChat;
use crate::models::telegram_subscription::TelegramSubscription;
use chrono::offset::FixedOffset;
use chrono::{DateTime, Utc};
use diesel::result::Error;
use handlebars::{to_json, Handlebars};
use html2text::from_read;
use htmlescape::decode_html;
use serde_json::value::Map;
use std::time::Duration;
use tokio::time;

static BLOCKED_ERROR: &str = "Forbidden: bot was blocked by the user";
static CHAT_NOT_FOUND: &str = "Bad Request: chat not found";
static KICKED_ERROR: &str = "Forbidden: bot was kicked from the supergroup chat";
static DEACTIVATED_ERROR: &str = "Forbidden: user is deactivated";
static CHAT_UPGRADED_ERROR: &str = "Bad Request: group chat was upgraded to a supergroup chat";
static BOT_IS_NOT_MEMBER: &str = "Forbidden: bot is not a member of the supergroup chat";
static BOT_IS_NOT_IN_CHANNEL: &str = "Forbidden: bot is not a member of the channel chat";
static BOT_IS_KICKED: &str = "Forbidden: bot was kicked from the channel chat";
static BOT_IS_KICKED_GROUP: &str = "Forbidden: bot was kicked from the group chat";

static DISCRIPTION_LIMIT: usize = 3500;

pub struct DeliverJob {}

#[derive(Debug)]
pub struct DeliverJobError {
    pub msg: String,
}

impl From<Error> for DeliverJobError {
    fn from(error: Error) -> Self {
        let msg = format!("{:?}", error);

        DeliverJobError { msg }
    }
}

impl DeliverJob {
    pub fn new() -> Self {
        DeliverJob {}
    }

    pub async fn execute(&self) -> Result<(), DeliverJobError> {
        let semaphored_connection = db::get_semaphored_connection().await;
        let db_connection = semaphored_connection.connection;
        let mut current_chats: Vec<i64>;
        let mut page = 1;
        let mut total_number = 0;

        log::info!("Started delivering feed items");

        loop {
            current_chats = telegram::fetch_chats_with_subscriptions(&db_connection, page, 1000)?;

            page += 1;

            if current_chats.is_empty() {
                break;
            }

            total_number += current_chats.len();

            for chat_id in current_chats {
                tokio::spawn(deliver_updates_for_chat(chat_id));
            }
        }

        log::info!("Started checking delivery for {} chats", total_number);

        Ok(())
    }
}

async fn deliver_updates_for_chat(chat_id: i64) -> Result<(), DeliverJobError> {
    let semaphored_connection = db::get_semaphored_connection().await;
    let connection = semaphored_connection.connection;
    let subscriptions = telegram::find_subscriptions_for_chat(&connection, chat_id)?;

    for subscription in subscriptions {
        match deliver_subscription_updates(&subscription).await {
            Ok(()) => (),
            Err(error) => log::error!(
                "Failed to deliver updates for subscription: {:?} {:?}",
                subscription,
                error
            ),
        }
    }

    Ok(())
}

async fn deliver_subscription_updates(
    subscription: &TelegramSubscription,
) -> Result<(), DeliverJobError> {
    let semaphored_connection = db::get_semaphored_connection().await;
    let connection = semaphored_connection.connection;
    let feed_items = telegram::find_undelivered_feed_items(&connection, &subscription)?;
    let undelivered_count = telegram::count_undelivered_feed_items(&connection, &subscription);
    let chat_id = subscription.chat_id;
    let feed = feeds::find(&connection, subscription.feed_id).unwrap();

    let chat = telegram::find_chat(&connection, chat_id).unwrap();
    let delay = delay_period(&chat);

    if feed_items.len() < undelivered_count as usize {
        let message = format!(
            "You have {} unread items, below {} last items for {}",
            undelivered_count,
            feed_items.len(),
            feed.link
        );

        match api::send_message(chat_id, message).await {
            Ok(_) => {
                time::delay_for(delay).await;
                ()
            }
            Err(error) => {
                let error_message = format!("{}", error);

                log::error!("Failed to deliver updates: {} {}", chat_id, error_message);

                if bot_blocked(&error_message) {
                    match telegram::remove_chat(&connection, chat_id) {
                        Ok(_) => log::info!("Successfully removed chat {}", chat_id),
                        Err(error) => log::error!("Failed to remove a chat {}", error),
                    }
                };

                return Err(DeliverJobError {
                    msg: format!("Failed to send updates : {}", error),
                });
            }
        }
    }

    if !feed_items.is_empty() {
        let offset = match chat.utc_offset_minutes {
            None => FixedOffset::west(0),
            Some(value) => {
                if value > 0 {
                    FixedOffset::east(value * 60)
                } else {
                    FixedOffset::west(-value * 60)
                }
            }
        };

        let template = match subscription.template.clone() {
            Some(template) => Some(template),
            None => chat.template,
        };

        let mut messages = format_messages(template, offset, feed_items.clone(), feed);
        messages.reverse();

        for message in messages {
            match api::send_message(chat_id, message).await {
                Ok(_) => {
                    time::delay_for(delay).await;
                    ()
                }
                Err(error) => {
                    let error_message = format!("{}", error);

                    log::error!("Failed to deliver updates: {}", error_message);

                    if bot_blocked(&error_message) {
                        match telegram::remove_chat(&connection, chat_id) {
                            Ok(_) => log::info!("Successfully removed chat {}", chat_id),
                            Err(error) => log::error!("Failed to remove a chat {}", error),
                        }
                    };

                    return Err(DeliverJobError {
                        msg: format!("Failed to send updates : {}", error),
                    });
                }
            };
        }

        match telegram::set_subscription_last_delivered_at(
            &connection,
            &subscription,
            get_max_publication_date(feed_items),
        ) {
            Ok(_) => (),
            Err(error) => {
                log::error!("Failed to set last_delivered_at: {}", error);
                return Err(DeliverJobError {
                    msg: format!("Failed to set last_delivered_at : {}", error),
                });
            }
        }
    }

    Ok(())
}

fn format_messages(
    template: Option<String>,
    date_offset: FixedOffset,
    feed_items: Vec<FeedItem>,
    feed: Feed,
) -> Vec<String> {
    let mut data = Map::new();

    let templ = match template {
        Some(t) => t,
        None => "{{bot_feed_name}}\n\n{{bot_item_name}}\n\n{{bot_date}}\n\n{{bot_item_link}}\n\n"
            .to_string(),
    };

    let reg = Handlebars::new();

    let feed_title = match feed.title {
        Some(title) => {
            let feed_title = truncate(&title, 50);

            feed_title
        }
        None => "".to_string(),
    };

    data.insert(
        "bot_feed_name".to_string(),
        to_json(remove_html(feed_title)),
    );
    data.insert("bot_feed_link".to_string(), to_json(feed.link));

    feed_items
        .iter()
        .map(|item| {
            let date = item.publication_date.with_timezone(&date_offset);

            data.insert("bot_date".to_string(), to_json(format!("{}", date)));
            data.insert(
                "bot_item_name".to_string(),
                to_json(remove_html(item.title.clone())),
            );

            data.insert("bot_item_link".to_string(), to_json(item.link.clone()));

            data.insert(
                "bot_item_description".to_string(),
                to_json(remove_html(item.description.clone().map_or_else(
                    || "".to_string(),
                    |s| truncate(&s, DISCRIPTION_LIMIT).to_string(),
                ))),
            );

            match reg.render_template(&templ, &data) {
                Err(error) => {
                    log::error!("Failed to render template {:?}", error);
                    "Failed to render a message".to_string()
                }
                Ok(result) => match decode_html(&result) {
                    Err(error) => {
                        log::error!("Failed to render template {:?}", error);
                        "Failed to render a message".to_string()
                    }
                    Ok(string) => string,
                },
            }
        })
        .collect::<Vec<String>>()
}

fn remove_html(string: String) -> String {
    from_read(&string.as_bytes()[..], 2000).trim().to_string()
}

fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => String::from(s),
        Some((idx, _)) => {
            let mut string = String::from(&s[..idx]);

            string.push_str("...");

            string
        }
    }
}

fn bot_blocked(error_message: &str) -> bool {
    error_message == BLOCKED_ERROR
        || error_message == CHAT_NOT_FOUND
        || error_message == KICKED_ERROR
        || error_message == DEACTIVATED_ERROR
        || error_message == BOT_IS_NOT_MEMBER
        || error_message == BOT_IS_KICKED
        || error_message == BOT_IS_KICKED_GROUP
        || error_message == BOT_IS_NOT_IN_CHANNEL
        || error_message.contains(CHAT_UPGRADED_ERROR)
}

fn get_max_publication_date(items: Vec<FeedItem>) -> DateTime<Utc> {
    items
        .into_iter()
        .max_by(|item1, item2| item1.publication_date.cmp(&item2.publication_date))
        .unwrap()
        .publication_date
}

fn delay_period(chat: &TelegramChat) -> Duration {
    match chat.kind.as_str() {
        "group" | "supergroup" => Duration::from_millis(2050),
        _ => Duration::from_millis(35),
    }
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::models::feed::Feed;
    use crate::models::feed_item::FeedItem;
    use chrono::offset::FixedOffset;
    use chrono::{DateTime, Utc};

    #[test]
    fn get_max_publication_date_finds_max_publication_date_in_feed_items_vector() {
        let feed_item1 = FeedItem {
            feed_id: 1,
            title: "".to_string(),
            description: None,
            link: "dsd".to_string(),
            author: None,
            guid: None,
            publication_date: DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into(),
            created_at: db::current_time(),
            updated_at: db::current_time(),
        };

        let feed_item2 = FeedItem {
            feed_id: 1,
            title: "".to_string(),
            description: None,
            link: "dsd1".to_string(),
            author: None,
            guid: None,

            publication_date: DateTime::parse_from_rfc2822("Wed, 13 May 2020 13:54:02 EDT")
                .unwrap()
                .into(),
            created_at: db::current_time(),
            updated_at: db::current_time(),
        };

        let feed_items = vec![feed_item1, feed_item2];
        let result = super::get_max_publication_date(feed_items);

        let expected_result: DateTime<Utc> =
            DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into();

        assert!(result == expected_result);
    }

    #[test]
    fn format_messages_uses_default_template_if_custom_template_is_missing() {
        let feed_items = vec![FeedItem {
            feed_id: 1,
            title: "Title".to_string(),
            description: Some("Description".to_string()),
            link: "dsd".to_string(),
            author: None,
            guid: None,
            publication_date: DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into(),
            created_at: db::current_time(),
            updated_at: db::current_time(),
        }];
        let feed = Feed {
            id: 1,
            title: Some("FeedTitle".to_string()),
            link: "link".to_string(),
            error: None,
            description: None,
            synced_at: None,
            created_at: db::current_time(),
            updated_at: db::current_time(),
            feed_type: "rss".to_string(),
        };

        let result = super::format_messages(None, FixedOffset::east(5 * 60), feed_items, feed);

        assert_eq!(
            result[0],
            "FeedTitle\n\nTitle\n\n2020-05-13 19:59:02 +00:05\n\ndsd\n\n".to_string()
        );
    }

    #[test]
    fn format_messages_uses_custom_template() {
        let feed_items = vec![FeedItem {
            feed_id: 1,
            title: "Title".to_string(),
            description: Some("Description".to_string()),
            link: "dsd".to_string(),
            author: None,
            guid: None,
            publication_date: DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into(),
            created_at: db::current_time(),
            updated_at: db::current_time(),
        }];
        let feed = Feed {
            id: 1,
            title: Some("FeedTitle".to_string()),
            link: "link".to_string(),
            error: None,
            description: None,
            synced_at: None,
            created_at: db::current_time(),
            updated_at: db::current_time(),
            feed_type: "rss".to_string(),
        };

        let result = super::format_messages(Some("{{bot_feed_name}} {{bot_feed_link}} {{bot_date}} {{bot_item_link}} {{bot_item_description}} {{bot_item_name}} {{bot_item_name}}".to_string()), FixedOffset::east(600 * 60), feed_items, feed);

        assert_eq!(
            result[0],
            "FeedTitle link 2020-05-14 05:54:02 +10:00 dsd Description Title Title".to_string()
        );
    }
}
