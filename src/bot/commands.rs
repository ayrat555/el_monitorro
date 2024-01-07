use crate::bot::telegram_client;
use crate::bot::telegram_client::Api;
use crate::bot::SimpleMessageParams;
use crate::config::Config;
use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::NewTelegramChat;
use crate::db::telegram::NewTelegramSubscription;
use crate::models::Feed;
use crate::models::TelegramChat;
use crate::models::TelegramSubscription;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::Chat;
use frankenstein::ChatType;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::LinkPreviewOptions;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use std::fmt;
use std::str::FromStr;
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub use close::Close;
pub use commands_keyboard::CommandsKeyboard;
pub use get_filter::GetFilter;
pub use get_global_filter::GetGlobalFilter;
pub use get_global_template::GetGlobalTemplate;
pub use get_preview_enabled::GetPreviewEnabled;
pub use get_template::GetTemplate;
pub use get_timezone::GetTimezone;
pub use help::Help;
pub use help_command_info::HelpCommandInfo;
pub use info::Info;
pub use list_subscriptions_keyboard::ListSubscriptionsKeyboard;
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
pub use show_feed_keyboard::ShowFeedKeyboard;
pub use start::Start;
pub use subscribe::Subscribe;
pub use toggle_preview_enabled::TogglePreviewEnabled;
pub use unknown_command::UnknownCommand;
pub use unsubscribe::Unsubscribe;

pub mod close;
pub mod commands_keyboard;
pub mod get_filter;
pub mod get_global_filter;
pub mod get_global_template;
pub mod get_preview_enabled;
pub mod get_template;
pub mod get_timezone;
pub mod help;
pub mod help_command_info;
pub mod info;
pub mod list_subscriptions_keyboard;
pub mod remove_filter;
pub mod remove_global_filter;
pub mod remove_global_template;
pub mod remove_template;
pub mod set_content_fields;
pub mod set_filter;
pub mod set_global_filter;
pub mod set_global_template;
pub mod set_template;
pub mod set_timezone;
pub mod show_feed_keyboard;
pub mod start;
pub mod subscribe;
pub mod toggle_preview_enabled;
pub mod unknown_command;
pub mod unsubscribe;

impl From<Chat> for NewTelegramChat {
    fn from(chat: Chat) -> Self {
        let kind = match chat.type_field {
            ChatType::Private => "private",
            ChatType::Group => "group",
            ChatType::Supergroup => "supergroup",
            ChatType::Channel => "channel",
        };

        NewTelegramChat {
            id: chat.id,
            kind: kind.to_string(),
            username: chat.username,
            first_name: chat.first_name,
            last_name: chat.last_name,
            title: chat.title,
        }
    }
}

#[derive(Debug)]
pub enum BotCommand {
    Close,
    CommandsKeyboard,
    GetFilter(String),
    GetGlobalFilter,
    GetGlobalTemplate,
    GetPreviewEnabled,
    GetTemplate(String),
    GetTimezone,
    Help,
    HelpCommandInfo(String),
    Info,
    ListSubscriptions,
    RemoveFilter(String),
    RemoveGlobalFilter,
    RemoveGlobalTemplate,
    RemoveTemplate(String),
    SetContentFields(String),
    SetFilter(String),
    SetGlobalFilter(String),
    SetGlobalTemplate(String),
    SetTemplate(String),
    SetTimezone(String),
    ShowFeedKeyboard(String),
    Start,
    Subscribe(String),
    TogglePreviewEnabled,
    UnknownCommand(String),
    Unsubscribe(String),
}

impl FromStr for BotCommand {
    type Err = ();

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let bot_command = if !command.starts_with('/') {
            BotCommand::UnknownCommand(command.to_string())
        } else if command.starts_with(CommandsKeyboard::command()) {
            BotCommand::CommandsKeyboard
        } else if command.starts_with(HelpCommandInfo::command()) {
            let args = parse_args(HelpCommandInfo::command(), command);

            BotCommand::HelpCommandInfo(args)
        } else if command.starts_with(Help::command()) {
            BotCommand::Help
        } else if command.starts_with(Subscribe::command()) {
            let args = parse_args(Subscribe::command(), command);

            BotCommand::Subscribe(args)
        } else if command.starts_with(Unsubscribe::command()) {
            let args = parse_args(Unsubscribe::command(), command);

            BotCommand::Unsubscribe(args)
        } else if command.starts_with(ListSubscriptionsKeyboard::command()) {
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
        } else if command.starts_with(ShowFeedKeyboard::command()) {
            let args = parse_args(ShowFeedKeyboard::command(), command);

            BotCommand::ShowFeedKeyboard(args)
        } else if command.starts_with(Close::command()) {
            BotCommand::Close
        } else if command.starts_with(GetPreviewEnabled::command()) {
            BotCommand::GetPreviewEnabled
        } else if command.starts_with(TogglePreviewEnabled::command()) {
            BotCommand::TogglePreviewEnabled
        } else {
            BotCommand::UnknownCommand(command.to_string())
        };

        Ok(bot_command)
    }
}

#[derive(Debug)]
pub enum ArgBotCommand {
    SetContentFields,
    SetFilter(String),
    SetGlobalFilter,
    SetGlobalTemplate,
    SetTemplate(String),
    SetTimezone,
    Subscribe,
    Cancel,
}

impl FromStr for ArgBotCommand {
    type Err = ();

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        if !command.starts_with('/') {
            Err(())
        } else if command.starts_with(Subscribe::command()) {
            Ok(ArgBotCommand::Subscribe)
        } else if command.starts_with(SetTimezone::command()) {
            Ok(ArgBotCommand::SetTimezone)
        } else if command.starts_with(SetFilter::command()) {
            let args = parse_args(SetFilter::command(), command);

            Ok(ArgBotCommand::SetFilter(args))
        } else if command.starts_with(SetTemplate::command()) {
            let args = parse_args(SetTemplate::command(), command);

            Ok(ArgBotCommand::SetTemplate(args))
        } else if command.starts_with(SetGlobalFilter::command()) {
            Ok(ArgBotCommand::SetGlobalFilter)
        } else if command.starts_with(SetGlobalTemplate::command()) {
            Ok(ArgBotCommand::SetGlobalTemplate)
        } else if command.starts_with(SetContentFields::command()) {
            Ok(ArgBotCommand::SetContentFields)
        } else if command.starts_with("/cancel") {
            Ok(ArgBotCommand::Cancel)
        } else {
            Err(())
        }
    }
}

impl fmt::Display for ArgBotCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ArgBotCommand::SetContentFields => write!(f, "OK. Send me content fields"),
            ArgBotCommand::SetFilter(_) => write!(f, "OK. Send me filter words"),
            ArgBotCommand::SetGlobalFilter => write!(f, "OK. Send me global filter words"),
            ArgBotCommand::SetGlobalTemplate => write!(f, "OK. Send me global template"),
            ArgBotCommand::SetTemplate(_) => write!(f, "OK. Send me template"),
            ArgBotCommand::SetTimezone => write!(f, "OK. Send me timezone in minutes"),
            ArgBotCommand::Subscribe => write!(f, "OK. Send me a feed url"),
            ArgBotCommand::Cancel => write!(f, "The command was cancelled"),
        }
    }
}

fn parse_args(command: &str, command_with_args: &str) -> String {
    let handle = Config::telegram_bot_handle();
    let command_with_handle = format!("{command}@{handle}");

    if command_with_args.starts_with(&command_with_handle) {
        command_with_args
            .replace(&command_with_handle, "")
            .trim()
            .to_string()
    } else {
        command_with_args.replace(command, "").trim().to_string()
    }
}

pub enum Response {
    Simple(String),
    Params(Box<SendMessageParams>),
}

pub trait Command {
    fn response(&self) -> Response;

    fn execute(&self, message: &Message, command: &str) {
        info!("{:?} wrote: {}", message.chat.id, command);

        match self.response() {
            Response::Simple(raw_message) => self.reply_to_message(message, raw_message),
            Response::Params(params) => self.send_message(*params),
        }
    }

    fn reply_to_message(&self, message: &Message, text: String) {
        let message_params = SimpleMessageParams::builder()
            .message(text)
            .chat_id(message.chat.id)
            .message_thread_id(message.message_thread_id)
            .build();

        if let Err(error) = self.api().reply_with_text_message(&message_params) {
            error!("Failed to reply to a message {:?} {:?}", error, message);
        }
    }

    fn send_message_and_remove(&self, send_message_params: SendMessageParams, message: &Message) {
        match self.api().send_message_with_params(&send_message_params) {
            Err(error) => {
                error!(
                    "Failed to send a message {:?} {:?}",
                    error, send_message_params
                );
            }

            Ok(_) => {
                self.remove_message(message);
            }
        }
    }

    fn send_message(&self, send_message_params: SendMessageParams) {
        if let Err(error) = self.api().send_message_with_params(&send_message_params) {
            error!(
                "Failed to send a message {:?} {:?}",
                error, send_message_params
            );
        }
    }

    fn remove_message(&self, message: &Message) {
        self.api().remove_message(message)
    }

    fn simple_keyboard(&self, text: String, back_command: String, message: &Message) -> Response {
        let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        let mut row: Vec<InlineKeyboardButton> = Vec::new();

        let button = InlineKeyboardButton::builder()
            .text("◀ Back")
            .callback_data(back_command)
            .build();

        row.push(button);
        buttons.push(row);
        buttons.push(Close::button_row());

        let keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(buttons)
            .build();

        let preview_params = LinkPreviewOptions::builder().is_disabled(true).build();

        let mut params = SendMessageParams::builder()
            .chat_id(message.chat.id)
            .link_preview_options(preview_params)
            .text(text)
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            .build();

        params.message_thread_id = message.message_thread_id;

        Response::Params(Box::new(params))
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
        feed_url_or_external_id: &str,
    ) -> Result<TelegramSubscription, String> {
        match Uuid::from_str(feed_url_or_external_id) {
            Ok(uuid) => match telegram::find_subscription_by_external_id(db_connection, uuid) {
                Some(subscription) => Ok(subscription),
                None => Err("Subscription does not exist".to_string()),
            },
            Err(_) => {
                self.find_subscription_by_feed_url(db_connection, chat_id, feed_url_or_external_id)
            }
        }
    }

    fn find_subscription_by_feed_url(
        &self,
        db_connection: &mut PgConnection,
        chat_id: i64,
        feed_url: &str,
    ) -> Result<TelegramSubscription, String> {
        let not_exists_error = Err("Subscription does not exist".to_string());
        let feed = self.find_feed(db_connection, feed_url)?;

        match self.find_subscription_by_chat_id_and_feed_id(db_connection, chat_id, feed.id) {
            Some(subscription) => Ok(subscription),
            None => not_exists_error,
        }
    }

    fn find_subscription_by_chat_id_and_feed_id(
        &self,
        db_connection: &mut PgConnection,
        chat_id: i64,
        feed_id: i64,
    ) -> Option<TelegramSubscription> {
        let telegram_subscription = NewTelegramSubscription::builder()
            .chat_id(chat_id)
            .feed_id(feed_id)
            .build();

        telegram::find_subscription(db_connection, telegram_subscription)
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
            let err = format!("The number of filter words is limited by {filter_limit}");
            return Err(err);
        }

        Ok(filter_words)
    }
}

#[derive(TypedBuilder)]
pub struct CommandProcessor {
    message: Message,

    #[builder(setter(into))]
    text: String,

    callback: bool,
}

impl CommandProcessor {
    pub fn process(&mut self) {
        if let ChatType::Private = self.message.chat.type_field {
            self.process_private_chat_command();
        } else {
            self.process_regular_command();
        }
    }

    fn process_private_chat_command(&mut self) {
        let mut connection = crate::db::pool().get().unwrap();

        match telegram::find_chat(&mut connection, self.message.chat.id) {
            None => match ArgBotCommand::from_str(&self.text) {
                Ok(command) => {
                    let chat =
                        telegram::create_chat(&mut connection, (*self.message.chat.clone()).into())
                            .unwrap();

                    self.save_command(&mut connection, &chat, command);
                }
                Err(_) => {
                    self.process_regular_command();
                }
            },

            Some(chat) => match &chat.command {
                None => match ArgBotCommand::from_str(&self.text) {
                    Ok(command) => {
                        self.save_command(&mut connection, &chat, command);
                    }
                    Err(_) => self.process_regular_command(),
                },
                Some(command) => {
                    if let Ok(command) = ArgBotCommand::from_str(&self.text) {
                        return self.save_command(&mut connection, &chat, command);
                    }

                    if let Ok(BotCommand::UnknownCommand(_)) = BotCommand::from_str(&self.text) {
                        let full_command = format!("{command} {}", self.text);
                        self.text = full_command;

                        self.process_regular_command();
                    } else {
                        self.process_regular_command();
                    }

                    telegram::set_command(&mut connection, &chat, None).unwrap();
                }
            },
        }
    }

    fn save_command(
        &self,
        connection: &mut PgConnection,
        chat: &TelegramChat,
        command: ArgBotCommand,
    ) {
        match command {
            ArgBotCommand::Cancel => telegram::set_command(connection, chat, None).unwrap(),

            _ => telegram::set_command(connection, chat, Some(self.text.clone())).unwrap(),
        };

        let message_params = SimpleMessageParams::builder()
            .message(command.to_string())
            .chat_id(self.message.chat.id)
            .build();

        info!("{:?} wrote: {}", self.message.chat.id, self.text);

        if let Err(error) = telegram_client::api().reply_with_text_message(&message_params) {
            log::error!("Failed to send a message: {error:?}")
        }
    }

    fn process_regular_command(&self) {
        match BotCommand::from_str(&self.text).unwrap() {
            BotCommand::CommandsKeyboard => {
                if let ChatType::Private = self.message.chat.type_field {
                    CommandsKeyboard::builder()
                        .message(self.message.clone())
                        .build()
                        .run()
                } else {
                    UnknownCommand::builder()
                        .message(self.message.clone())
                        .args("/commands".to_string())
                        .build()
                        .run()
                }
            }

            BotCommand::Subscribe(args) => Subscribe::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::Help => Help::builder().message(self.message.clone()).build().run(),

            BotCommand::Unsubscribe(args) => Unsubscribe::builder()
                .message(self.message.clone())
                .args(args)
                .callback(self.callback)
                .build()
                .run(),

            BotCommand::ListSubscriptions => ListSubscriptionsKeyboard::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::Start => Start::builder().message(self.message.clone()).build().run(),

            BotCommand::SetTimezone(args) => SetTimezone::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::GetTimezone => GetTimezone::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::SetFilter(args) => SetFilter::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::GetFilter(args) => GetFilter::builder()
                .message(self.message.clone())
                .args(args)
                .callback(self.callback)
                .build()
                .run(),

            BotCommand::RemoveFilter(args) => RemoveFilter::builder()
                .message(self.message.clone())
                .args(args)
                .callback(self.callback)
                .build()
                .run(),

            BotCommand::SetTemplate(args) => SetTemplate::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::GetTemplate(args) => GetTemplate::builder()
                .message(self.message.clone())
                .args(args)
                .callback(self.callback)
                .build()
                .run(),

            BotCommand::RemoveTemplate(args) => RemoveTemplate::builder()
                .message(self.message.clone())
                .args(args)
                .callback(self.callback)
                .build()
                .run(),

            BotCommand::SetGlobalTemplate(args) => SetGlobalTemplate::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::RemoveGlobalTemplate => RemoveGlobalTemplate::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::GetGlobalTemplate => GetGlobalTemplate::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::SetGlobalFilter(args) => SetGlobalFilter::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::GetGlobalFilter => GetGlobalFilter::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::RemoveGlobalFilter => RemoveGlobalFilter::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::Info => Info::builder().message(self.message.clone()).build().run(),

            BotCommand::SetContentFields(args) => SetContentFields::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::UnknownCommand(args) => UnknownCommand::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::HelpCommandInfo(args) => HelpCommandInfo::builder()
                .message(self.message.clone())
                .args(args)
                .build()
                .run(),

            BotCommand::Close => Close::builder().message(self.message.clone()).build().run(),

            BotCommand::GetPreviewEnabled => GetPreviewEnabled::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::TogglePreviewEnabled => TogglePreviewEnabled::builder()
                .message(self.message.clone())
                .build()
                .run(),

            BotCommand::ShowFeedKeyboard(args) => ShowFeedKeyboard::builder()
                .message(self.message.clone())
                .feed_url_or_external_id(args)
                .build()
                .run(),
        };
    }
}
