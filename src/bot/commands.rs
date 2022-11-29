use crate::bot::commands::list_subscriptions_inline_keyboard::ListSubscriptionsInlineKeyboard;
use crate::bot::commands::set_global_template_inline_keyboard::SetGlobalTemplateInlineKeyboard;
use crate::bot::commands::set_template_inline_keyboard::SetTemplateInlineKeyboard;
use crate::bot::commands::subscribe_inline_keyboard::SubscribeInlineKeyboard;
use crate::bot::commands::unsubscribe_inline_keyboard::UnsubscribeInlineKeyboard;
use crate::bot::telegram_client;
use crate::bot::telegram_client::api;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds;
use crate::db::pool;
use crate::db::telegram;
use crate::db::telegram::NewTelegramChat;
use crate::db::telegram::NewTelegramSubscription;
use crate::models::Feed;
use crate::models::TelegramSubscription;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::Chat;
use frankenstein::ChatType;
use frankenstein::DeleteMessageParams;
use frankenstein::Message;
use frankenstein::TelegramApi;
use std::str::FromStr;

pub use get_filter::GetFilter;
pub use get_global_filter::GetGlobalFilter;
pub use get_global_template::GetGlobalTemplate;
pub use get_template::GetTemplate;
pub use get_timezone::GetTimezone;
pub use help::Help;
pub use info::Info;
pub use list_subscriptions::ListSubscriptions;
pub use remove_filter::RemoveFilter;
pub use remove_global_filter::RemoveGlobalFilter;
pub use remove_global_template::RemoveGlobalTemplate;
pub use remove_template::RemoveTemplate;
pub use set_content_fields::SetContentFields;
pub use set_filter::SetFilter;
pub use set_global_filter::SetGlobalFilter;
pub use set_global_template::SetGlobalTemplate;
pub use set_template::SetTemplate;
pub use set_timezone::SetTimezone;
pub use start::Start;
pub use subscribe::Subscribe;
pub use unknown_command::UnknownCommand;
pub use unsubscribe::Unsubscribe;

pub mod get_filter;
pub mod get_global_filter;
pub mod get_global_template;
pub mod get_template;
pub mod get_timezone;
pub mod help;
pub mod info;
pub mod list_subscriptions;
pub mod list_subscriptions_inline_keyboard;
pub mod remove_filter;
pub mod remove_global_filter;
pub mod remove_global_template;
pub mod remove_template;
pub mod set_content_fields;
pub mod set_filter;
pub mod set_global_filter;
pub mod set_global_template;
pub mod set_global_template_inline_keyboard;
pub mod set_template;
pub mod set_template_inline_keyboard;
pub mod set_timezone;
pub mod start;
pub mod subscribe;
pub mod subscribe_inline_keyboard;
pub mod unknown_command;
pub mod unsubscribe;
pub mod unsubscribe_inline_keyboard;

impl From<Chat> for NewTelegramChat {
    fn from(chat: Chat) -> Self {
        let kind = match chat.type_field {
            ChatType::Private => "private",
            ChatType::Group => "group",
            ChatType::Supergroup => "supergroup",
            ChatType::Channel => "channel",
        };

        NewTelegramChat {
            id: chat.id as i64,
            kind: kind.to_string(),
            username: chat.username,
            first_name: chat.first_name,
            last_name: chat.last_name,
            title: chat.title,
        }
    }
}

pub enum BotCommand {
    UnknownCommand(String),
    Help,
    Subscribe(String),
    Unsubscribe(String),
    ListSubscriptions,
    Start,
    SetTimezone(String),
    GetTimezone,
    SetFilter(String),
    GetFilter(String),
    RemoveFilter(String),
    SetTemplate(String),
    GetTemplate(String),
    RemoveTemplate(String),
    GetGlobalFilter,
    SetGlobalFilter(String),
    RemoveGlobalFilter,
    GetGlobalTemplate,
    SetGlobalTemplate(String),
    RemoveGlobalTemplate,
    Info,
    SetContentFields(String),
}

impl FromStr for BotCommand {
    type Err = ();

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let bot_command = if !command.starts_with('/') {
            BotCommand::UnknownCommand(command.to_string())
        } else if command.starts_with(Help::command()) {
            BotCommand::Help
        } else if command.starts_with(Subscribe::command()) {
            let args = parse_args(Subscribe::command(), command);

            BotCommand::Subscribe(args)
        } else if command.starts_with(Unsubscribe::command()) {
            let args = parse_args(Unsubscribe::command(), command);

            BotCommand::Unsubscribe(args)
        } else if command.starts_with(ListSubscriptions::command()) {
            BotCommand::ListSubscriptions
        } else if command.starts_with(Start::command()) {
            BotCommand::Start
        } else if command.starts_with(SetTimezone::command()) {
            let args = parse_args(SetTimezone::command(), command);

            BotCommand::SetTimezone(args)
        } else if command.starts_with(GetTimezone::command()) {
            BotCommand::GetTimezone
        } else if command.starts_with(SetFilter::command()) {
            let args = parse_args(SetFilter::command(), command);

            BotCommand::SetFilter(args)
        } else if command.starts_with(GetFilter::command()) {
            let args = parse_args(GetFilter::command(), command);

            BotCommand::GetFilter(args)
        } else if command.starts_with(RemoveFilter::command()) {
            let args = parse_args(RemoveFilter::command(), command);

            BotCommand::RemoveFilter(args)
        } else if command.starts_with(SetTemplate::command()) {
            let args = parse_args(SetTemplate::command(), command);

            BotCommand::SetTemplate(args)
        } else if command.starts_with(GetTemplate::command()) {
            let args = parse_args(GetTemplate::command(), command);

            BotCommand::GetTemplate(args)
        } else if command.starts_with(RemoveTemplate::command()) {
            let args = parse_args(RemoveTemplate::command(), command);

            BotCommand::RemoveTemplate(args)
        } else if command.starts_with(SetGlobalFilter::command()) {
            let args = parse_args(SetGlobalFilter::command(), command);

            BotCommand::SetGlobalFilter(args)
        } else if command.starts_with(RemoveGlobalTemplate::command()) {
            BotCommand::RemoveGlobalTemplate
        } else if command.starts_with(GetGlobalTemplate::command()) {
            BotCommand::GetGlobalTemplate
        } else if command.starts_with(SetGlobalTemplate::command()) {
            let args = parse_args(SetGlobalTemplate::command(), command);

            BotCommand::SetGlobalTemplate(args)
        } else if command.starts_with(GetGlobalFilter::command()) {
            BotCommand::GetGlobalFilter
        } else if command.starts_with(RemoveGlobalFilter::command()) {
            BotCommand::RemoveGlobalFilter
        } else if command.starts_with(Info::command()) {
            BotCommand::Info
        } else if command.starts_with(SetContentFields::command()) {
            let args = parse_args(SetContentFields::command(), command);

            BotCommand::SetContentFields(args)
        } else {
            BotCommand::UnknownCommand(command.to_string())
        };

        Ok(bot_command)
    }
}

pub fn parse_args(command: &str, command_with_args: &str) -> String {
    let handle = Config::telegram_bot_handle();
    let command_with_handle = format!("{}@{}", command, handle);

    if command_with_args.starts_with(&command_with_handle) {
        command_with_args
            .replace(&command_with_handle, "")
            .trim()
            .to_string()
    } else {
        command_with_args.replace(command, "").trim().to_string()
    }
}

pub trait Command {
    fn response(&self) -> String;

    fn execute(&self, message: &Message) {
        info!(
            "{:?} wrote: {}",
            message.chat.id,
            message.text.as_ref().unwrap()
        );

        let text: &str = message.text.as_ref().unwrap();
        let command = BotCommand::from_str(text).unwrap();

        let data = match self.fetch_db_connection() {
            Ok(mut connection) => self.list_subscriptions(&mut connection, message.clone()),
            Err(_error_message) => "error fetching data".to_string(),
        };

        let feed_id = match self.fetch_db_connection() {
            Ok(mut connection) => self.list_feed_id(&mut connection, message),
            Err(_error_message) => "error fetching data".to_string(),
        };

        let feeds = data.split('\n');
        let feeds_ids = feed_id.split(',').clone();

        let texts = self.response();
        let delete_message_params = DeleteMessageParams::builder()
            .chat_id(message.chat.id)
            .message_id(message.message_id)
            .build();

        match command {
            BotCommand::Subscribe(args) => match args.is_empty() {
                true => {
                    let send_message_params =
                        SubscribeInlineKeyboard::set_subscribe_keyboard(message);
                    api().send_message(&send_message_params).unwrap();
                }
                false => self.reply_to_message(message, texts),
            },
            BotCommand::Unsubscribe(args) => match args.is_empty() {
                true => {
                    api().delete_message(&delete_message_params).unwrap();
                    let send_message_params = UnsubscribeInlineKeyboard::set_unsubscribe_keyboard(
                        message, feeds, feed_id,
                    );
                    api().send_message(&send_message_params).unwrap();
                }
                false => self.reply_to_message(message, texts),
            },
            BotCommand::SetGlobalTemplate(args) => match args.is_empty() {
                true => {
                    api().delete_message(&delete_message_params).unwrap();
                    let send_message_params =
                        SetGlobalTemplateInlineKeyboard::set_global_template_keyboard(message);
                    api().send_message(&send_message_params).unwrap();
                }
                false => self.reply_to_message(message, texts),
            },
            BotCommand::ListSubscriptions => {
                let send_message_params =
                    ListSubscriptionsInlineKeyboard::select_feed_url_keyboard_list_subscriptions(
                        message,
                        feeds_ids,
                        pool().to_owned(),
                    );
                api().send_message(&send_message_params).unwrap();
            }
            BotCommand::SetTemplate(args) => match args.is_empty() {
                true => {
                    let send_message_params = SetTemplateInlineKeyboard::select_feed_url_keyboard(
                        message,
                        feeds_ids,
                        pool().to_owned(),
                    );
                    api().send_message(&send_message_params).unwrap();
                }
                false => self.reply_to_message(message, texts),
            },
            _ => {
                self.reply_to_message(message, texts);
            }
        };
    }

    fn reply_to_message(&self, message: &Message, text: String) {
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
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, String> {
        match crate::db::pool().get() {
            Ok(connection) => Ok(connection),
            Err(err) => {
                error!("Failed to fetch a connection from the pool {:?}", err);

                Err("Failed to process your command. Please contact @Ayrat555".to_string())
            }
        }
    }

    fn api(&self) -> Api {
        telegram_client::api().clone()
    }

    fn find_subscription(
        &self,
        db_connection: &mut PgConnection,
        chat_id: i64,
        feed_url: &str,
    ) -> Result<TelegramSubscription, String> {
        let not_exists_error = Err("Subscription does not exist".to_string());
        let feed = self.find_feed(db_connection, feed_url)?;

        let chat = match telegram::find_chat(db_connection, chat_id) {
            Some(chat) => chat,
            None => return not_exists_error,
        };

        let telegram_subscription = NewTelegramSubscription {
            chat_id: chat.id,
            feed_id: feed.id,
        };

        match telegram::find_subscription(db_connection, telegram_subscription) {
            Some(subscription) => Ok(subscription),
            None => not_exists_error,
        }
    }

    fn find_feed(&self, db_connection: &mut PgConnection, feed_url: &str) -> Result<Feed, String> {
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
