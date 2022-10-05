use super::unknown_command::UnknownCommand;
use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/info";

#[derive(TypedBuilder)]
pub struct Info {
    message: Message,
}

impl Info {
    pub fn run(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        self.execute(db_pool, api, message);
    }

    fn info(&self, db_connection: &mut PgConnection) -> String {
        let total_feeds = match feeds::count_feeds_with_subscriptions(db_connection) {
            Ok(res) => res,
            Err(err) => {
                log::error!("Failed to fetch total feeds count {:?}", err);
                return "Failed to fetch total feeds count".to_string();
            }
        };

        let total_chats = match telegram::count_chats_with_subscriptions(db_connection) {
            Ok(res) => res,
            Err(err) => {
                log::error!("Failed to fetch total chats count {:?}", err);
                return "Failed to fetch total chats count".to_string();
            }
        };

        let mut result_message = format!(
            "the number of feeds is {}\n\
             the number of chats is {} \n",
            total_feeds, total_chats
        );

        for kind in ["private", "group", "supergroup", "channel"] {
            let result = match telegram::count_chats_of_type(db_connection, kind) {
                Ok(res) => res,
                Err(err) => {
                    log::error!("Failed to fetch {} chats count {:?}", kind, err);
                    return "Failed to fetch chats count".to_string();
                }
            };

            result_message = format!("{}\n{} chats - {}", result_message, kind, result);
        }

        result_message
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    fn unknown_command(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        api: Api,
        message: Message,
    ) {
        UnknownCommand::builder()
            .message(self.message.clone())
            .args(self.message.text.clone().unwrap())
            .build()
            .run(db_pool, api, message);
    }
}

impl Command for Info {
    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        match Config::admin_telegram_id() {
            None => self.unknown_command(db_pool, api, message),
            Some(id) => {
                if id == message.chat.id {
                    info!(
                        "{:?} wrote: {}",
                        message.chat.id,
                        message.text.as_ref().unwrap()
                    );

                    let text = self.response();

                    self.reply_to_message(message, text)
                } else {
                    self.unknown_command(db_pool, api, message);
                }
            }
        }
    }

    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.info(&mut connection),
            Err(error_message) => error_message,
        }
    }

    fn reply_to_message(&self, message: Message, text: String) {
        if let Err(error) =
            &self
                .api()
                .reply_with_text_message(message.chat.id, text, Some(message.message_id))
        {
            error!("Failed to reply to update {:?} {:?}", error, message);
        }
    }

    fn fetch_db_connection(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>, String> {
        match crate::db::pool().get() {
            Ok(connection) => Ok(connection),
            Err(err) => {
                error!("Failed to fetch a connection from the pool {:?}", err);

                Err("Failed to process your command. Please contact @Ayrat555".to_string())
            }
        }
    }

    fn api(&self) -> Api {
        crate::bot::telegram_client::api().clone()
    }

    fn find_subscription(
        &self,
        db_connection: &mut PgConnection,
        chat_id: i64,
        feed_url: &str,
    ) -> Result<crate::models::TelegramSubscription, String> {
        let not_exists_error = Err("Subscription does not exist".to_string());
        let feed = self.find_feed(db_connection, feed_url)?;

        let chat = match telegram::find_chat(db_connection, chat_id) {
            Some(chat) => chat,
            None => return not_exists_error,
        };

        let telegram_subscription = telegram::NewTelegramSubscription {
            chat_id: chat.id,
            feed_id: feed.id,
        };

        match telegram::find_subscription(db_connection, telegram_subscription) {
            Some(subscription) => Ok(subscription),
            None => not_exists_error,
        }
    }

    fn find_feed(
        &self,
        db_connection: &mut PgConnection,
        feed_url: &str,
    ) -> Result<crate::models::Feed, String> {
        match feeds::find_by_link(db_connection, feed_url) {
            Some(feed) => Ok(feed),
            None => Err("Feed does not exist".to_string()),
        }
    }

    fn parse_filter(&self, params: &str) -> Result<Vec<String>, String> {
        let filter_words: Vec<String> =
            params.split(',').map(|s| s.trim().to_lowercase()).collect();

        let filter_limit = Config::filter_limit();

        if filter_words.len() > filter_limit {
            let err = format!("The number of filter words is limited by {}", filter_limit);
            return Err(err);
        }

        Ok(filter_words)
    }

    fn list_subscriptions(&self, db_connection: &mut PgConnection, message: Message) -> String {
        match telegram::find_feeds_by_chat_id(db_connection, message.chat.id) {
            Err(_) => "Couldn't fetch your subscriptions".to_string(),
            Ok(feeds) => {
                if feeds.is_empty() {
                    "You don't have any subscriptions".to_string()
                } else {
                    feeds
                        .into_iter()
                        .map(|feed| feed.link)
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            }
        }
    }

    fn list_feed_id(&self, db_connection: &mut PgConnection, message: &Message) -> String {
        match telegram::find_feeds_by_chat_id(db_connection, message.chat.id) {
            Err(_) => "Couldn't fetch your subscriptions".to_string(),
            Ok(feeds) => {
                if feeds.is_empty() {
                    "You don't have any subscriptions".to_string()
                } else {
                    feeds
                        .into_iter()
                        .map(|feed| feed.id.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                }
            }
        }
    }
}
