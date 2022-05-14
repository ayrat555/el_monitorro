use crate::bot::telegram_client::Api;
use crate::db::feeds;
use crate::db::telegram;
use crate::models::feed::Feed;
use crate::models::feed_item::FeedItem;
use crate::models::telegram_chat::TelegramChat;
use crate::models::telegram_subscription::TelegramSubscription;
use chrono::offset::FixedOffset;
use chrono::{DateTime, Utc};
use diesel::result::Error;
use fang::typetag;
use fang::Error as FangError;
use fang::PgConnection;
use fang::Runnable;
use handlebars::{to_json, Handlebars};
use htmlescape::decode_html;
use serde::{Deserialize, Serialize};
use serde_json::value::Map;
use std::time::Duration;

const TELEGRAM_ERRORS: [&str; 14] = [
    "Bad Request: CHAT_WRITE_FORBIDDEN",
    "Bad Request: chat not found",
    "Bad Request: group chat was upgraded to a supergroup chat",
    "Bad Request: group chat was upgraded to a supergroup chat, migrate to chat id",
    "Bad Request: have no rights to send a message",
    "Bad Request: need administrator rights in the channel chat",
    "Forbidden: bot is not a member of the channel chat",
    "Forbidden: bot is not a member of the supergroup chat",
    "Forbidden: bot was blocked by the user",
    "Forbidden: bot was kicked from the channel chat",
    "Forbidden: bot was kicked from the group chat",
    "Forbidden: bot was kicked from the supergroup chat",
    "Forbidden: the group chat was deleted",
    "Forbidden: user is deactivated",
];

const DISCRIPTION_LIMIT: usize = 2500;
const UNICODE_EMPTY_CHARS: [char; 5] = ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'];
const HTML_SPACE: &str = "&#32;";
const MESSAGES_LIMIT: i64 = 10;
const JOB_TYPE: &str = "deliver";

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

#[derive(Serialize, Deserialize)]
pub struct DeliverChatUpdatesJob {
    pub chat_id: i64,
}

impl DeliverChatUpdatesJob {
    pub fn new(chat_id: i64) -> Self {
        Self { chat_id }
    }

    pub fn deliver(&self, db_connection: &PgConnection) {
        let subscriptions =
            telegram::find_unread_subscriptions_for_chat(db_connection, self.chat_id).unwrap();
        let api = Api::default();

        for subscription in subscriptions {
            match self.deliver_subscription_updates(&subscription, db_connection, &api) {
                Ok(()) => {
                    telegram::mark_subscription_delivered(db_connection, &subscription).unwrap();
                }

                Err(error) => {
                    log::error!(
                        "Failed to deliver updates for subscription: {:?} {:?}",
                        subscription,
                        error
                    );
                    break;
                }
            }
        }
    }

    fn deliver_subscription_updates(
        &self,
        subscription: &TelegramSubscription,
        connection: &PgConnection,
        api: &Api,
    ) -> Result<(), DeliverJobError> {
        let feed_items =
            telegram::find_undelivered_feed_items(connection, subscription, MESSAGES_LIMIT)?;

        let chat_id = subscription.chat_id;
        let feed = feeds::find(connection, subscription.feed_id).unwrap();

        let chat = telegram::find_chat(connection, chat_id).unwrap();

        self.maybe_send_unread_messages_count(
            subscription,
            connection,
            feed_items.len() as i64,
            feed.link.clone(),
            api,
            &chat,
        )?;

        if !feed_items.is_empty() {
            let template = match subscription.template.clone() {
                Some(template) => Some(template),
                None => chat.template.clone(),
            };

            let messages = format_messages(template, chat.utc_offset_minutes, feed_items, feed);

            match subscription.filter_words.clone() {
                None => {
                    for (message, publication_date) in messages {
                        self.send_text_message_and_updated_subscription(
                            subscription,
                            message,
                            connection,
                            &chat,
                            api,
                            publication_date,
                        )?
                    }
                }
                Some(words) => self.send_messages_with_filter(
                    words,
                    messages,
                    connection,
                    subscription,
                    api,
                    &chat,
                )?,
            }
        }

        Ok(())
    }

    fn send_messages_with_filter(
        &self,
        words: Vec<String>,
        messages: Vec<(String, DateTime<Utc>)>,
        connection: &PgConnection,
        subscription: &TelegramSubscription,
        api: &Api,
        chat: &TelegramChat,
    ) -> Result<(), DeliverJobError> {
        let (negated_words, regular_words): (Vec<String>, Vec<String>) =
            words.into_iter().partition(|word| word.starts_with('!'));

        for (message, publication_date) in messages {
            let mut mtch = true;

            if !regular_words.is_empty() {
                let regular_mtch = regular_words
                    .iter()
                    .any(|word| message.to_lowercase().contains(word));

                mtch = regular_mtch;
            }

            if !negated_words.is_empty() {
                let negated_mtch = negated_words.iter().all(|neg_word| {
                    let word = neg_word.replace('!', "");

                    !message.to_lowercase().contains(&word)
                });

                mtch = mtch && negated_mtch;
            }

            if mtch {
                self.send_text_message_and_updated_subscription(
                    subscription,
                    message,
                    connection,
                    chat,
                    api,
                    publication_date,
                )?;
            } else {
                self.update_last_deivered_at(connection, subscription, publication_date)?;
            }
        }

        Ok(())
    }

    fn maybe_send_unread_messages_count(
        &self,
        subscription: &TelegramSubscription,
        connection: &PgConnection,
        feed_items_count: i64,
        feed_link: String,
        api: &Api,
        chat: &TelegramChat,
    ) -> Result<(), DeliverJobError> {
        let undelivered_count = telegram::count_undelivered_feed_items(connection, subscription);

        if chat.kind == "channel" {
            return Ok(());
        }

        if subscription.filter_words.is_some() {
            return Ok(());
        }

        if feed_items_count == MESSAGES_LIMIT && undelivered_count > MESSAGES_LIMIT {
            let message = format!(
                "You have {} unread items, below {} last items for {}",
                undelivered_count, feed_items_count, feed_link
            );

            self.send_text_message(chat, message, connection, api)?;
        }

        Ok(())
    }

    fn send_text_message(
        &self,
        chat: &TelegramChat,
        message: String,
        connection: &PgConnection,
        api: &Api,
    ) -> Result<(), DeliverJobError> {
        let delay = delay_period(chat);

        match api.send_text_message(chat.id, message) {
            Ok(_) => {
                std::thread::sleep(delay);
                Ok(())
            }

            Err(error) => {
                let error_message = format!("{:?}", error);

                Err(handle_error(error_message, connection, chat.id))
            }
        }
    }

    fn send_text_message_and_updated_subscription(
        &self,
        subscription: &TelegramSubscription,
        message: String,
        connection: &PgConnection,
        chat: &TelegramChat,
        api: &Api,
        publication_date: DateTime<Utc>,
    ) -> Result<(), DeliverJobError> {
        self.send_text_message(chat, message, connection, api)?;

        self.update_last_deivered_at(connection, subscription, publication_date)
    }

    fn update_last_deivered_at(
        &self,
        connection: &PgConnection,
        subscription: &TelegramSubscription,
        publication_date: DateTime<Utc>,
    ) -> Result<(), DeliverJobError> {
        match telegram::set_subscription_last_delivered_at(
            connection,
            subscription,
            publication_date,
        ) {
            Ok(_) => Ok(()),
            Err(error) => {
                log::error!("Failed to set last_delivered_at: {}", error);
                Err(DeliverJobError {
                    msg: format!("Failed to set last_delivered_at : {}", error),
                })
            }
        }
    }
}

#[typetag::serde]
impl Runnable for DeliverChatUpdatesJob {
    fn run(&self, connection: &PgConnection) -> Result<(), FangError> {
        self.deliver(connection);

        Ok(())
    }

    fn task_type(&self) -> String {
        JOB_TYPE.to_string()
    }
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

fn format_messages(
    template: Option<String>,
    offset: Option<i32>,
    feed_items: Vec<FeedItem>,
    feed: Feed,
) -> Vec<(String, DateTime<Utc>)> {
    let time_offset = match offset {
        None => FixedOffset::west(0),
        Some(value) => {
            if value > 0 {
                FixedOffset::east(value * 60)
            } else {
                FixedOffset::west(-value * 60)
            }
        }
    };

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

    let mut formatted_messages = feed_items
        .iter()
        .map(|item| {
            let date = item.publication_date.with_timezone(&time_offset);

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
                    Ok(string) => (truncate_and_check(&string, 4000), item.created_at),
                },
            }
        })
        .collect::<Vec<(String, DateTime<Utc>)>>();

    formatted_messages.reverse();

    formatted_messages
}

fn remove_html(string_with_maybe_html: String) -> String {
    let text = nanohtml2text::html2text(&string_with_maybe_html);

    truncate(&text, 2000).trim().to_string()
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

    remove_empty_characters(trimmed_result)
}

fn truncate_and_check(s: &str, max_chars: usize) -> String {
    let truncated_result = truncate(s, max_chars);

    if truncated_result.is_empty() {
        "According to your template the message is empty. Telegram doesn't support empty messages. That's why we're sending this placeholder message.".to_string()
    } else {
        truncated_result
    }
}

fn remove_empty_characters(string: &str) -> String {
    let mut result = string.to_string();
    for character in UNICODE_EMPTY_CHARS {
        result = result.replace(character, "");
    }

    result.replace(HTML_SPACE, "")
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
    use crate::models::Feed;
    use crate::models::FeedItem;
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
            content_hash: "".to_string(),
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
            sync_retries: 0,
            sync_skips: 0,
            content_fields: None,
        };

        let result = super::format_messages(None, Some(5), feed_items, feed);

        assert_eq!(
            result[0].0,
            "FeedTitle\n\nTitle\n\nDescription\n\n2020-05-13 19:59:02 +00:05\n\ndsd".to_string()
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
            content_hash: "".to_string(),
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
            sync_retries: 0,
            sync_skips: 0,
            content_fields: None,
        };

        let result = super::format_messages(Some("{{bot_feed_name}} {{bot_feed_link}} {{bot_date}} {{bot_item_link}} {{bot_item_description}} {{bot_item_name}} {{bot_item_name}}".to_string()), Some(600), feed_items, feed);

        assert_eq!(
            result[0].0,
            "FeedTitle link 2020-05-14 05:54:02 +10:00 dsd Description Title Title".to_string()
        );

        assert_eq!(result[0].1, current_time);
    }

    #[test]
    fn removes_empty_unicode_characters() {
        let publication_date: DateTime<Utc> =
            DateTime::parse_from_rfc2822("Wed, 13 May 2020 15:54:02 EDT")
                .unwrap()
                .into();
        let current_time = db::current_time();

        let feed = Feed {
            id: 1,
            title: Some("".to_string()),
            link: "".to_string(),
            error: None,
            description: None,
            synced_at: None,
            created_at: db::current_time(),
            updated_at: db::current_time(),
            feed_type: "".to_string(),
            sync_retries: 0,
            sync_skips: 0,
            content_fields: None,
        };

        let feed_items = vec![FeedItem {
            publication_date,
            feed_id: 1,
            title: "".to_string(),
            description: Some("\u{200b}".to_string()),
            link: "".to_string(),
            author: None,
            guid: None,
            content_hash: "".to_string(),
            created_at: current_time,
            updated_at: current_time,
        }];

        let result = super::format_messages(Some("{{bot_feed_name}} {{bot_feed_link}} {{bot_item_link}} {{bot_item_description}} {{bot_item_name}} {{bot_item_name}}".to_string()), Some(60), feed_items, feed);

        assert_eq!(
            result[0].0,
            "According to your template the message is empty. Telegram doesn't support empty messages. That's why we're sending this placeholder message.".to_string()
        );

        assert_eq!(result[0].1, current_time);
    }
}
