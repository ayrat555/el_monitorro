use super::MessageRenderer;
use crate::bot::telegram_client;
use crate::bot::telegram_client::Api;
use crate::bot::SimpleMessageParams;
use crate::db::feeds;
use crate::db::telegram;
use crate::models::Feed;
use crate::models::FeedItem;
use crate::models::TelegramChat;
use crate::models::TelegramSubscription;
use aho_corasick::AhoCorasickBuilder;
use chrono::{DateTime, Utc};
use diesel::result::Error;
use fang::typetag;
use fang::FangError;
use fang::PgConnection;
use fang::Queueable;
use fang::Runnable;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use typed_builder::TypedBuilder;

const TELEGRAM_ERRORS: [&str; 16] = [
    "Bad Request: CHAT_WRITE_FORBIDDEN",
    "Bad Request: TOPIC_CLOSED",
    "Bad Request: chat not found",
    "Bad Request: group chat was upgraded to a supergroup chat",
    "Bad Request: group chat was upgraded to a supergroup chat, migrate to chat id",
    "Bad Request: have no rights to send a message",
    "Bad Request: not enough rights to send text messages to the chat",
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

const MESSAGES_LIMIT: usize = 10;
const JOB_TYPE: &str = "deliver";

#[derive(Debug)]
pub struct DeliverJobError {
    pub msg: String,
}

impl From<Error> for DeliverJobError {
    fn from(error: Error) -> Self {
        let msg = format!("{error:?}");

        DeliverJobError { msg }
    }
}

#[derive(TypedBuilder)]
pub struct DeliverChatUpdates<'a> {
    chat: TelegramChat,
    feed: Feed,
    subscription: TelegramSubscription,
    db_connection: &'a mut PgConnection,
    api: &'a Api,
}

impl DeliverChatUpdates<'_> {
    pub fn deliver(&mut self) -> Result<(), DeliverJobError> {
        let feed_items = telegram::find_undelivered_feed_items(
            self.db_connection,
            &self.subscription,
            MESSAGES_LIMIT as i64,
        )?;

        if feed_items.is_empty() {
            return Ok(());
        }

        let filter_words = self.filter_words();

        if filter_words.is_none() {
            self.maybe_send_unread_messages_count(feed_items.len())?;
        }

        let formatted_messages = self.format_messages(feed_items);

        match self.filter_words() {
            None => self.send_messages_without_filter(formatted_messages),

            Some(words) => self.send_messages_with_filter(words, formatted_messages),
        }
    }

    fn filter_words(&self) -> Option<Vec<String>> {
        if self.chat.filter_words.is_some() {
            return self.chat.filter_words.clone();
        }

        self.subscription.filter_words.clone()
    }

    fn maybe_send_unread_messages_count(
        &mut self,
        feed_items_count: usize,
    ) -> Result<(), DeliverJobError> {
        let undelivered_count =
            telegram::count_undelivered_feed_items(self.db_connection, &self.subscription);

        if self.chat.kind == "channel" {
            return Ok(());
        }

        if feed_items_count == MESSAGES_LIMIT && undelivered_count > MESSAGES_LIMIT as i64 {
            let message = format!("You have {undelivered_count} unread items, below {feed_items_count} last items for {}", self.feed.link);

            self.send_text_message(message)?;
        }

        Ok(())
    }

    fn send_text_message(&mut self, message: String) -> Result<(), DeliverJobError> {
        let delay = self.delay_period();

        let message_params = SimpleMessageParams::builder()
            .message(message)
            .chat_id(self.chat.id)
            .preview_enabled(self.chat.preview_enabled)
            .message_thread_id(self.subscription.thread_id)
            .build();

        match self.api.reply_with_text_message(&message_params) {
            Ok(_) => {
                std::thread::sleep(delay);
                Ok(())
            }

            Err(error) => {
                let error_message = format!("{error:?}");

                Err(self.handle_error(error_message))
            }
        }
    }

    fn delay_period(&self) -> Duration {
        match self.chat.kind.as_str() {
            "group" | "supergroup" => Duration::from_millis(2200),
            _ => Duration::from_millis(35),
        }
    }

    fn handle_error(&mut self, error: String) -> DeliverJobError {
        log::error!("Failed to deliver updates: {}", error);

        if self.bot_blocked(&error) {
            match telegram::remove_chat(self.db_connection, self.chat.id) {
                Ok(_) => log::info!("Successfully removed chat {}", self.chat.id),
                Err(error) => log::error!("Failed to remove a chat {error}"),
            }
        };

        DeliverJobError {
            msg: format!("Failed to send updates : {error}"),
        }
    }

    fn bot_blocked(&self, error_message: &str) -> bool {
        TELEGRAM_ERRORS
            .iter()
            .any(|&message| error_message.contains(message))
    }

    fn format_messages(&self, feed_items: Vec<FeedItem>) -> Vec<(String, DateTime<Utc>)> {
        let template = match &self.subscription.template {
            Some(template) => Some(template.clone()),
            None => self.chat.template.clone(),
        };

        let message_renderer_builder = MessageRenderer::builder()
            .offset(self.chat.utc_offset_minutes)
            .template(template)
            .bot_feed_name(self.feed.title.clone())
            .bot_feed_link(Some(self.feed.link.clone()));

        let mut formatted_messages = feed_items
            .iter()
            .map(|item| {
                let message_renderer = message_renderer_builder
                    .clone()
                    .bot_date(item.publication_date)
                    .bot_item_name(item.title.clone())
                    .bot_item_link(item.link.clone())
                    .bot_item_description(item.description.clone())
                    .bot_item_author(item.author.clone())
                    .build();

                match message_renderer.render() {
                    Ok(message) => (message, item.created_at),
                    Err(error_message) => (error_message, item.created_at),
                }
            })
            .collect::<Vec<(String, DateTime<Utc>)>>();

        formatted_messages.reverse();

        formatted_messages
    }

    fn send_messages_without_filter(
        &mut self,
        messages: Vec<(String, DateTime<Utc>)>,
    ) -> Result<(), DeliverJobError> {
        for (message, publication_date) in messages {
            self.send_text_message_and_updated_subscription(message, publication_date)?
        }

        Ok(())
    }

    fn send_messages_with_filter(
        &mut self,
        words: Vec<String>,
        messages: Vec<(String, DateTime<Utc>)>,
    ) -> Result<(), DeliverJobError> {
        let (negated_words, regular_words): (Vec<String>, Vec<String>) =
            words.into_iter().partition(|word| word.starts_with('!'));

        let negated_words: Vec<String> = negated_words
            .into_iter()
            .map(|word| word.replace('!', ""))
            .collect();

        for (message, publication_date) in messages {
            let mut mtch = true;
            let lowercase_message = message.to_lowercase();

            if !regular_words.is_empty() {
                mtch = self.check_filter_words(&lowercase_message, &regular_words);
            }

            if !negated_words.is_empty() {
                let negated_mtch = self.check_filter_words(&lowercase_message, &negated_words);

                mtch = mtch && !negated_mtch;
            }

            if mtch {
                self.send_text_message_and_updated_subscription(message, publication_date)?;
            } else {
                self.update_last_deivered_at(publication_date)?;
            }
        }

        Ok(())
    }

    fn send_text_message_and_updated_subscription(
        &mut self,
        message: String,
        publication_date: DateTime<Utc>,
    ) -> Result<(), DeliverJobError> {
        self.send_text_message(message)?;

        self.update_last_deivered_at(publication_date)
    }

    fn update_last_deivered_at(
        &mut self,
        publication_date: DateTime<Utc>,
    ) -> Result<(), DeliverJobError> {
        match telegram::set_subscription_last_delivered_at(
            self.db_connection,
            &self.subscription,
            publication_date,
        ) {
            Ok(_) => Ok(()),
            Err(error) => {
                log::error!("Failed to set last_delivered_at: {error}");
                Err(DeliverJobError {
                    msg: format!("Failed to set last_delivered_at : {error}"),
                })
            }
        }
    }

    fn check_filter_words(&self, text: &str, words: &Vec<String>) -> bool {
        match AhoCorasickBuilder::new().build(words) {
            Ok(ac) => ac.find(text).is_some(),
            Err(error) => {
                log::error!("Failed to build aho-corasick: {error}");

                true
            }
        }
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

    pub fn deliver(&self, db_connection: &mut PgConnection) -> Result<(), FangError> {
        let chat = telegram::find_chat(db_connection, self.chat_id);

        if chat.is_none() {
            return Ok(());
        }

        let subscriptions =
            telegram::find_unread_subscriptions_for_chat(db_connection, self.chat_id)?;

        let api = telegram_client::api();

        for subscription in subscriptions {
            let feed = feeds::find(db_connection, subscription.feed_id);

            if feed.is_none() {
                continue;
            }

            let mut deliver_chat_updates = DeliverChatUpdates::builder()
                .chat(chat.clone().unwrap())
                .feed(feed.unwrap())
                .subscription(subscription.clone())
                .db_connection(db_connection)
                .api(api)
                .build();

            match deliver_chat_updates.deliver() {
                Ok(()) => {
                    telegram::mark_subscription_delivered(db_connection, &subscription)?;
                }

                Err(error) => {
                    log::error!(
                        "Failed to deliver updates for subscription: {subscription:?} {error:?}",
                    );
                    break;
                }
            }
        }
        Ok(())
    }
}

#[typetag::serde]
impl Runnable for DeliverChatUpdatesJob {
    fn run(&self, _queue: &dyn Queueable) -> Result<(), FangError> {
        let mut db_connection = crate::db::pool().get()?;

        self.deliver(&mut db_connection)
    }

    fn uniq(&self) -> bool {
        true
    }

    fn max_retries(&self) -> i32 {
        0
    }

    fn task_type(&self) -> String {
        JOB_TYPE.to_string()
    }
}
