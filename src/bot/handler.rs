use super::commands::get_filter::GetFilter;
use super::commands::get_global_filter::GetGlobalFilter;
use super::commands::get_global_template::GetGlobalTemplate;
use super::commands::get_template::GetTemplate;
use super::commands::get_timezone::GetTimezone;
use super::commands::help::Help;
use super::commands::info::Info;
use super::commands::list_subscriptions::ListSubscriptions;
use super::commands::list_subscriptions_inline_keyboard::ListSubscriptionsInlineKeyboard;
use super::commands::remove_filter::RemoveFilter;
use super::commands::remove_global_filter::RemoveGlobalFilter;
use super::commands::remove_global_template::RemoveGlobalTemplate;
use super::commands::remove_template::RemoveTemplate;
use super::commands::set_content_fields::SetContentFields;
use super::commands::set_filter::SetFilter;
use super::commands::set_global_filter::SetGlobalFilter;
use super::commands::set_global_template::SetGlobalTemplate;
use super::commands::set_template::SetTemplate;
use super::commands::set_timezone::SetTimezone;
use super::commands::start::Start;
use super::commands::subscribe::Subscribe;
use super::commands::unknown_command::UnknownCommand;
use super::commands::unsubscribe::Unsubscribe;
use regex::Regex;

use crate::bot::commands::set_global_template_inline_keyboard::SetGlobalTemplateInlineKeyboard;
use crate::bot::commands::set_template_inline_keyboard::SetTemplateInlineKeyboard;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds::find;

use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::DeleteMessageParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use frankenstein::UpdateContent;
use std::thread;

const DEFAULT_TEMPLATE: &str = "{{bot_feed_name}}\n\n{{bot_item_name}}\n\n{{bot_item_description}}\n\n{{bot_date}}\n\n{{bot_item_link}}\n\n";
pub struct Handler {}

impl Handler {
    pub fn start() {
        // maybe Api can be share also
        let mut api = Api::default();
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(Config::commands_db_pool_number() as usize)
            .build()
            .unwrap();

        log::info!("Starting the El Monitorro bot");

        let interval = std::time::Duration::from_secs(1);
        loop {
            while let Some(update) = api.next_update() {
                let db_pool = crate::db::pool().clone();
                let tg_api = api.clone();

                match update.content {
                    UpdateContent::Message(_) => {
                        thread_pool.spawn(move || {
                            Self::process_message_or_channel_post(db_pool, tg_api, update)
                        });
                    }
                    UpdateContent::ChannelPost(_) => {
                        thread_pool.spawn(move || {
                            Self::process_message_or_channel_post(db_pool, tg_api, update)
                        });
                    }
                    UpdateContent::CallbackQuery(_) => {
                        thread_pool
                            .spawn(move || Self::process_callback_query(db_pool, tg_api, update));
                    }
                    _ => return,
                }
            }

            thread::sleep(interval);
        }
    }

    fn process_message_or_channel_post(
        db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
        api: Api,
        update: Update,
    ) {
        let bot_name = Config::telegram_bot_name();
        let message = match update.content {
            UpdateContent::Message(message) => message,
            UpdateContent::ChannelPost(channel_post) => channel_post,
            _ => return,
        };

        if let Some(owner_id) = Self::owner_telegram_id() {
            if message.from.is_none() {
                return;
            }

            if message.from.as_ref().unwrap().id as i64 != owner_id {
                return;
            }
        }

        let text = message.text.clone();

        if text.is_none() {
            return;
        }

        let _delete_message_params = DeleteMessageParams::builder()
            .chat_id(message.chat.id)
            .message_id(message.message_id)
            .build();

        let commands = &text.unwrap();
        let command = &commands.replace(&bot_name, ""); //removes bot name from the command (switch_inline_query_current_chat adds botname automatically)

        if !command.starts_with('/') {
            UnknownCommand::execute(db_pool, api, message);
        } else if command.starts_with(Subscribe::command()) {
            Subscribe::execute(db_pool, api, message);
        } else if command.starts_with(Help::command()) {
            Help::execute(db_pool, api, message);
        } else if command.starts_with(Unsubscribe::command()) {
            Unsubscribe::execute(db_pool, api, message);
        } else if command.starts_with(ListSubscriptions::command()) {
            ListSubscriptions::execute(db_pool, api, message);
        } else if command.starts_with(Start::command()) {
            Start::execute(db_pool, api, message);
        } else if command.starts_with(SetTimezone::command()) {
            SetTimezone::execute(db_pool, api, message);
        } else if command.starts_with(GetTimezone::command()) {
            GetTimezone::execute(db_pool, api, message);
        } else if command.starts_with(SetFilter::command()) {
            SetFilter::execute(db_pool, api, message);
        } else if command.starts_with(GetFilter::command()) {
            GetFilter::execute(db_pool, api, message);
        } else if command.starts_with(RemoveFilter::command()) {
            RemoveFilter::execute(db_pool, api, message);
        } else if command.starts_with(SetTemplate::command()) {
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with(GetTemplate::command()) {
            GetTemplate::execute(db_pool, api, message);
        } else if command.starts_with(RemoveTemplate::command()) {
            RemoveTemplate::execute(db_pool, api, message);
        } else if command.starts_with(SetGlobalTemplate::command()) {
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with(RemoveGlobalTemplate::command()) {
            RemoveGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with(GetGlobalTemplate::command()) {
            GetGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with(SetGlobalFilter::command()) {
            SetGlobalFilter::execute(db_pool, api, message);
        } else if command.starts_with(GetGlobalFilter::command()) {
            GetGlobalFilter::execute(db_pool, api, message);
        } else if command.starts_with(RemoveGlobalFilter::command()) {
            RemoveGlobalFilter::execute(db_pool, api, message);
        } else if command.starts_with(Info::command()) {
            Info::execute(db_pool, api, message);
        } else if command.starts_with(SetContentFields::command()) {
            SetContentFields::execute(db_pool, api, message);
        } else {
            UnknownCommand::execute(db_pool, api, message);
        }
    }

    fn owner_telegram_id() -> Option<i64> {
        Config::owner_telegram_id()
    }

    fn process_callback_query(
        db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
        api: Api,
        update: Update,
    ) {
        let bot_name = Config::telegram_bot_name();
        let query = match update.content {
            UpdateContent::CallbackQuery(callback_query) => callback_query,
            _ => return,
        };
        let mut message = query.message.unwrap();
        let messageid = message.message_id;
        let chatid = message.chat.id;
        let text = query.data;
        let delete_message_params = DeleteMessageParams::builder()
            .chat_id(chatid)
            .message_id(messageid)
            .build();
        if text.is_none() {
            return;
        }

        let commands = &text.unwrap();
        let data = &commands.replace(&bot_name, "");
        message.text = Some(data.clone());

        let command = data;

        if command.starts_with(ListSubscriptions::command()) {
            ListSubscriptions::execute(db_pool, api, message);
        } else if command.starts_with(ListSubscriptions::callback()) {
            let feed_id = Self::parse_int_from_string(command);
            if feed_id == "none" {
                ListSubscriptions::execute(db_pool, api, message);
            } else {
                let feed_url = Self::get_feed_url_by_id(db_pool, feed_id.clone());
                api.delete_message(&delete_message_params).unwrap();
                let send_message_params =
                    ListSubscriptionsInlineKeyboard::set_list_subcriptions_menu_keyboard(
                        message, feed_id, feed_url,
                    );
                api.send_message(&send_message_params).unwrap();
            }
        } else if command.starts_with(GetFilter::command()) {
            let feed_url =
                Self::get_feed_url_by_id(db_pool.clone(), Self::parse_int_from_string(command));
            message.text = Some(format!("/get_filter {}", feed_url));
            GetFilter::execute(db_pool, api, message);
        } else if command.starts_with(SetTemplate::command()) {
            let feed_id = Self::parse_int_from_string(command);
            let feed_url = Self::get_feed_url_by_id(db_pool.clone(), feed_id.clone());
            let text = command.replace(&feed_id, &feed_url);
            message.text = Some(text.trim().to_string());
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with(SetTemplate::callback()) {
            let feed_id = Self::parse_int_from_string(command);
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params =
                SetTemplateInlineKeyboard::set_template_menu_keyboard(message, feed_id);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetTemplateInlineKeyboard::substring()) {
            api.delete_message(&delete_message_params).unwrap();
            let feed_id = Self::parse_int_from_string(command);
            let data = command.replace("substring", "");
            let feed_url = Self::get_feed_url_by_id(db_pool, feed_id);
            let send_message_params =
                SetTemplateInlineKeyboard::set_template_substring_keyboard(message, data, feed_url);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetTemplateInlineKeyboard::italic()) {
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("italic", "");
            let send_message_params =
                SetTemplateInlineKeyboard::set_template_italic_keyboard(message, data);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetTemplateInlineKeyboard::bold()) {
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("bold", "");
            let send_message_params =
                SetTemplateInlineKeyboard::set_template_bold_keyboard(message, data);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetTemplateInlineKeyboard::create_link()) {
            api.delete_message(&delete_message_params).unwrap();
            let feed_id = Self::parse_int_from_string(command);
            let data = command.replace("create_link", "");
            let feed_url = Self::get_feed_url_by_id(db_pool, feed_id);
            let send_message_params = SetTemplateInlineKeyboard::set_template_create_link_keyboard(
                message, data, feed_url,
            );
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetTemplate::default_template()) {
            let feed_url =
                Self::get_feed_url_by_id(db_pool.clone(), Self::parse_int_from_string(command));
            message.text = Some(format!("/set_template {} {}", feed_url, DEFAULT_TEMPLATE));
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with(GetTemplate::command()) {
            let feed_id = Self::parse_int_from_string(command);
            let feed_url = Self::get_feed_url_by_id(db_pool.clone(), feed_id.clone());
            let text = command.replace(&feed_id, &feed_url);
            message.text = Some(text.trim().to_string());
            GetTemplate::execute(db_pool, api, message);
        } else if command.starts_with(RemoveTemplate::command()) {
            let feed_id = Self::parse_int_from_string(command);
            let feed_url = Self::get_feed_url_by_id(db_pool.clone(), feed_id.clone());
            let text = command.replace(&feed_id, &feed_url);
            message.text = Some(text.trim().to_string());
            RemoveTemplate::execute(db_pool, api, message);
        } else if command.starts_with(RemoveFilter::command()) {
            let feed_id = Self::parse_int_from_string(command);
            let feed_url = Self::get_feed_url_by_id(db_pool.clone(), feed_id.clone());
            let text = command.replace(&feed_id, &feed_url);
            message.text = Some(text.trim().to_string());
            RemoveFilter::execute(db_pool, api, message);
        } else if command.starts_with(SetGlobalTemplate::command()) {
            match command.as_str() {
                "/set_global_template create_link_description" => {
                    message.text = Some(
                        "/set_global_template {{create_link bot_item_description bot_item_link}}"
                            .to_string(),
                    );
                    SetGlobalTemplate::execute(db_pool, api, message);
                }
                "/set_global_template create_link_item_name" => {
                    message.text = Some(
                        "/set_global_template {{create_link bot_item_name bot_item_link}}"
                            .to_string(),
                    );
                    SetGlobalTemplate::execute(db_pool, api, message)
                }
                _ => SetGlobalTemplate::execute(db_pool, api, message),
            }
        } else if command.starts_with(SetGlobalTemplateInlineKeyboard::italic()) {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params =
                SetGlobalTemplateInlineKeyboard::set_global_template_italic_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetGlobalTemplateInlineKeyboard::bold()) {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params =
                SetGlobalTemplateInlineKeyboard::set_global_template_bold_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetGlobalTemplateInlineKeyboard::create_link()) {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params =
                SetGlobalTemplateInlineKeyboard::set_global_template_create_link_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetGlobalTemplateInlineKeyboard::substring()) {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params =
                SetGlobalTemplateInlineKeyboard::set_global_template_substring_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with(SetGlobalTemplateInlineKeyboard::default_template()) {
            api.delete_message(&delete_message_params).unwrap();
            message.text = Some(format!("/set_global_template {}", DEFAULT_TEMPLATE));
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with(Unsubscribe::command()) {
            let feed_id = Self::parse_int_from_string(command);
            let feed_url = Self::get_feed_url_by_id(db_pool.clone(), feed_id.clone());
            let text = command.replace(&feed_id, &feed_url);
            message.text = Some(text.trim().to_string());
            Unsubscribe::execute(db_pool, api, message);
        } else if command.starts_with(Unsubscribe::callback()) {
            let feed_id = Self::parse_int_from_string(command);
            if feed_id == "none" {
                Unsubscribe::execute(db_pool, api, message);
            } else {
                let feed_url = Self::get_feed_url_by_id(db_pool.clone(), feed_id);
                message.text = Some(format!("/unsubscribe {}", feed_url));
                Unsubscribe::execute(db_pool, api, message);
            }
        } else if command.starts_with(ListSubscriptionsInlineKeyboard::back_to_menu()) {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params =
                SetGlobalTemplateInlineKeyboard::set_global_template_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else {
            UnknownCommand::execute(db_pool, api, message)
        }
    }

    fn parse_int_from_string(command: &str) -> String {
        let re = Regex::new(
            r"(?x)
                (?P<name>\d+)  # the name
            ",
        )
        .unwrap();
        let data = match re.captures(command) {
            Some(s) => s["name"].to_string(),
            None => "none".to_string(),
        };

        data
    }
    pub fn get_feed_url_by_id(
        db_pool: Pool<ConnectionManager<PgConnection>>,
        data: String,
    ) -> String {
        let feed_id: i64 = data.parse().unwrap();
        match Self::fetch_db_connection(db_pool) {
            Ok(mut connection) => {
                let feeds = find(&mut *connection, feed_id).unwrap();
                let data = feeds;
                data.link
            }
            Err(_error_message) => "error fetching message".to_string(),
        }
    }
    pub fn fetch_db_connection(
        db_pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, String> {
        match db_pool.get() {
            Ok(connection) => Ok(connection),
            Err(err) => {
                error!("Failed to fetch a connection from the pool {:?}", err);

                Err("Failed to process your command. Please contact @Ayrat555".to_string())
            }
        }
    }
}
