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
use diesel::pg::PgConnection;
use diesel::result::Error;
use fang::typetag;
use fang::Error as FangError;
use fang::Postgres;
use fang::Runnable;
use fang::{Deserialize, Serialize};
use handlebars::{to_json, Handlebars};
use html2text::from_read;
use htmlescape::decode_html;
use serde_json::value::Map;
use std::time::Duration;

static TELEGRAM_ERRORS: [&str; 13] = [
    "Forbidden: bot was blocked by the user",
    "Bad Request: chat not found",
    "Forbidden: bot was kicked from the supergroup chat",
    "Forbidden: user is deactivated",
    "Bad Request: group chat was upgraded to a supergroup chat",
    "Forbidden: bot is not a member of the supergroup chat",
    "Forbidden: bot is not a member of the channel chat",
    "Forbidden: bot was kicked from the channel chat",
    "Forbidden: bot was kicked from the group chat",
    "Bad Request: have no rights to send a message",
    "Bad Request: group chat was upgraded to a supergroup chat, migrate to chat id",
    "Bad Request: CHAT_WRITE_FORBIDDEN",
    "Bad Request: need administrator rights in the channel chat",
];

static DISCRIPTION_LIMIT: usize = 2500;

#[derive(Serialize, Deserialize)]
pub struct DeliverJob {}

impl Default for DeliverJob {
    fn default() -> Self {
        Self::new()
    }
}

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
}

#[typetag::serde]
impl Runnable for DeliverJob {
    fn run(&self) -> Result<(), FangError> {
        let postgres = Postgres::new();
        let mut current_chats: Vec<i64>;
        let mut page = 1;
        let mut total_chat_number = 0;

        log::info!("Started delivering feed items");

        loop {
            current_chats =
                match telegram::fetch_chats_with_subscriptions(&postgres.connection, page, 1000) {
                    Ok(chats) => chats,
                    Err(error) => {
                        let description = format!("{:?}", error);

                        return Err(FangError { description });
                    }
                };

            page += 1;

            if current_chats.is_empty() {
                break;
            }

            total_chat_number += current_chats.len();

            for chat_id in current_chats {
                postgres
                    .push_task(&DeliverChatUpdatesJob { chat_id })
                    .unwrap();
            }
        }

        log::info!("Started checking delivery for {} chats", total_chat_number,);

        Ok(())
    }

    fn task_type(&self) -> String {
        "deliver".to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeliverChatUpdatesJob {
    pub chat_id: i64,
}

impl DeliverChatUpdatesJob {
    pub fn deliver(&self) {
        let db_connection = db::establish_connection();
        let subscriptions =
            telegram::find_subscriptions_for_chat(&db_connection, self.chat_id).unwrap();

        for subscription in subscriptions {
            match deliver_subscription_updates(&subscription, &db_connection) {
                Ok(()) => (),
                Err(error) => log::error!(
                    "Failed to deliver updates for subscription: {:?} {:?}",
                    subscription,
                    error
                ),
            }
        }
    }
}

#[typetag::serde]
impl Runnable for DeliverChatUpdatesJob {
    fn run(&self) -> Result<(), FangError> {
        self.deliver();

        Ok(())
    }

    fn task_type(&self) -> String {
        "deliver".to_string()
    }
}

fn deliver_subscription_updates(
    subscription: &TelegramSubscription,
    connection: &PgConnection,
) -> Result<(), DeliverJobError> {
    let feed_items = telegram::find_undelivered_feed_items(connection, subscription)?;
    let undelivered_count = telegram::count_undelivered_feed_items(connection, subscription);
    let chat_id = subscription.chat_id;
    let feed = feeds::find(connection, subscription.feed_id).unwrap();

    let chat = telegram::find_chat(connection, chat_id).unwrap();
    let delay = delay_period(&chat);

    if subscription.filter_words.is_none() && feed_items.len() < undelivered_count as usize {
        let message = format!(
            "You have {} unread items, below {} last items for {}",
            undelivered_count,
            feed_items.len(),
            feed.link
        );

        match api::send_message_sync(chat_id, message) {
            Ok(_) => {
                std::thread::sleep(delay);
            }

            Err(error) => {
                let error_message = format!("{:?}", error);

                return Err(handle_error(error_message, connection, chat_id));
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

        let mut messages = format_messages(template, offset, feed_items, feed);
        messages.reverse();

        for (message, publication_date) in messages {
            match subscription.filter_words.clone() {
                None => match api::send_message_sync(chat_id, message) {
                    Ok(_) => {
                        std::thread::sleep(delay);
                        update_last_deivered_at(connection, subscription, publication_date)?;
                    }
                    Err(error) => {
                        let error_message = format!("{:?}", error);

                        return Err(handle_error(error_message, connection, chat_id));
                    }
                },
                Some(words) => {
                    let (negated_words, regular_words): (Vec<String>, Vec<String>) =
                        words.into_iter().partition(|word| word.starts_with('!'));

                    let mut mtch = true;

                    if !regular_words.is_empty() {
                        let regular_mtch = regular_words
                            .iter()
                            .any(|word| message.to_lowercase().contains(word));

                        mtch = regular_mtch;
                    }

                    if !negated_words.is_empty() {
                        let negated_mtch = negated_words.iter().all(|neg_word| {
                            let word = neg_word.replace("!", "");

                            !message.to_lowercase().contains(&word)
                        });

                        mtch = mtch && negated_mtch;
                    }

                    if mtch {
                        match api::send_message_sync(chat_id, message) {
                            Ok(_) => {
                                std::thread::sleep(delay);
                                update_last_deivered_at(
                                    connection,
                                    subscription,
                                    publication_date,
                                )?;
                            }
                            Err(error) => {
                                let error_message = format!("{:?}", error);

                                return Err(handle_error(error_message, connection, chat_id));
                            }
                        }
                    } else {
                        update_last_deivered_at(connection, subscription, publication_date)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_error(error: String, connection: &PgConnection, chat_id: i64) -> DeliverJobError {
    log::error!("Failed to deliver updates: {}", error);

    if bot_blocked(&error) {
        match telegram::remove_chat(connection, chat_id) {
            Ok(_) => log::info!("Successfully removed chat {}", chat_id),
            Err(error) => log::error!("Failed to remove a chat {}", error),
        }
    };

    DeliverJobError {
        msg: format!("Failed to send updates : {}", error),
    }
}

fn update_last_deivered_at(
    connection: &PgConnection,
    subscription: &TelegramSubscription,
    publication_date: DateTime<Utc>,
) -> Result<(), DeliverJobError> {
    match telegram::set_subscription_last_delivered_at(connection, subscription, publication_date) {
        Ok(_) => Ok(()),
        Err(error) => {
            log::error!("Failed to set last_delivered_at: {}", error);
            Err(DeliverJobError {
                msg: format!("Failed to set last_delivered_at : {}", error),
            })
        }
    }
}

fn format_messages(
    template: Option<String>,
    date_offset: FixedOffset,
    feed_items: Vec<FeedItem>,
    feed: Feed,
) -> Vec<(String, DateTime<Utc>)> {
    let mut data = Map::new();

    let templ = match template {
        Some(t) => t,
        None => "{{bot_feed_name}}\n\n{{bot_item_name}}\n\n{{bot_item_description}}\n\n{{bot_date}}\n\n{{bot_item_link}}\n\n"
            .to_string(),
    };

    let reg = Handlebars::new();

    let feed_title = match feed.title {
        Some(title) => truncate(&title, 50),
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
                    |s| truncate(&s, DISCRIPTION_LIMIT),
                ))),
            );

            match reg.render_template(&templ, &data) {
                Err(error) => {
                    log::error!("Failed to render template {:?}", error);
                    ("Failed to render a message".to_string(), item.created_at)
                }
                Ok(result) => match decode_html(&result) {
                    Err(error) => {
                        log::error!("Failed to render template {:?}", error);
                        ("Failed to render a message".to_string(), item.created_at)
                    }
                    Ok(string) => (truncate(&string, 4000), item.created_at),
                },
            }
        })
        .collect::<Vec<(String, DateTime<Utc>)>>()
}

fn remove_html(string: String) -> String {
    from_read(string.as_bytes(), 2000).trim().to_string()
}

fn truncate(s: &str, max_chars: usize) -> String {
    let result = match s.char_indices().nth(max_chars) {
        None => String::from(s),
        Some((idx, _)) => {
            let mut string = String::from(&s[..idx]);

            string.push_str("...");

            string
        }
    };

    let trimmed_result = result.trim();

    if trimmed_result.is_empty() {
        "According to your template the message is empty. Telegram doesn't support empty messages. That's why we're sending this placeholder message.".to_string()
    } else {
        trimmed_result.to_string()
    }
}

fn bot_blocked(error_message: &str) -> bool {
    TELEGRAM_ERRORS
        .iter()
        .any(|&message| error_message.contains(message))
}

fn delay_period(chat: &TelegramChat) -> Duration {
    match chat.kind.as_str() {
        "group" | "supergroup" => Duration::from_millis(2200),
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
    fn format_messages_uses_default_template_if_custom_template_is_missing() {
        let publication_date: DateTime<Utc> =
            DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into();
        let feed_items = vec![FeedItem {
            publication_date,
            feed_id: 1,
            title: "Title".to_string(),
            description: Some("Description".to_string()),
            link: "dsd".to_string(),
            author: None,
            guid: None,
            created_at: publication_date,
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
            result[0].0,
            "FeedTitle\n\nTitle\n\nDescription\n\n2020-05-13 19:59:02 +00:05\n\ndsd\n\n"
                .to_string()
        );

        assert_eq!(result[0].1, publication_date);
    }

    #[test]
    fn format_messages_uses_custom_template() {
        let publication_date: DateTime<Utc> =
            DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into();
        let current_time = db::current_time();
        let feed_items = vec![FeedItem {
            publication_date,
            feed_id: 1,
            title: "Title".to_string(),
            description: Some("Description".to_string()),
            link: "dsd".to_string(),
            author: None,
            guid: None,
            created_at: current_time,
            updated_at: current_time,
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
            result[0].0,
            "FeedTitle link 2020-05-14 05:54:02 +10:00 dsd Description Title Title".to_string()
        );

        assert_eq!(result[0].1, current_time);
    }
}
