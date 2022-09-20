use super::commands::get_filter::GetFilter;
use super::commands::get_global_filter::GetGlobalFilter;
use super::commands::get_global_template::GetGlobalTemplate;
use super::commands::get_template::GetTemplate;
use super::commands::get_timezone::GetTimezone;
use super::commands::help::Help;
use super::commands::info::Info;
use super::commands::list_subscriptions::ListSubscriptions;
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

use crate::bot::commands::list_subscriptions::set_list_subcriptions_menu_keyboard;
use crate::bot::commands::set_global_template::set_global_template_bold_keyboard;
use crate::bot::commands::set_global_template::set_global_template_create_link_keyboard;
use crate::bot::commands::set_global_template::set_global_template_italic_keyboard;
use crate::bot::commands::set_global_template::set_global_template_keyboard;
use crate::bot::commands::set_global_template::set_global_template_substring_keyboard;

use crate::bot::commands::set_template::set_template_bold_keyboard;
use crate::bot::commands::set_template::set_template_create_link_keyboard;
use crate::bot::commands::set_template::set_template_italic_keyboard;
use crate::bot::commands::set_template::set_template_menu_keyboard;
use crate::bot::commands::set_template::set_template_substring_keyboard;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds::find;
use crate::db::telegram;

use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::DeleteMessageParams;
use frankenstein::Message;
use frankenstein::TelegramApi;
use frankenstein::Update;
use frankenstein::UpdateContent;
use std::thread;

const BOT_NAME: &str = "@sasaathulbot "; 
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

                match update.content.clone() {
                    UpdateContent::Message(ref _message) => {
                        thread_pool.spawn(move || {
                            Self::process_message_or_channel_post(db_pool, tg_api, update)
                        });
                    }
                    UpdateContent::ChannelPost(ref _channelpost) => {
                        thread_pool.spawn(move || {
                            Self::process_message_or_channel_post(db_pool, tg_api, update)
                        });
                    }
                    UpdateContent::CallbackQuery(ref _callback_query) => {
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

        let commands = &text.unwrap();
        let _delete_message_params = DeleteMessageParams::builder()
            .chat_id(message.chat.id)
            .message_id(message.message_id)
            .build();

        let command = &commands.replace(BOT_NAME, ""); //removes bot name from the command (switch_inline_query_current_chat adds botname automatically)

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
        let query = match update.content {
            UpdateContent::CallbackQuery(callback_query) => callback_query,
            _ => return,
        };
        let mut message = query.message.unwrap();
        let _data = match fetch_db_connection(db_pool.clone()) {
            Ok(mut connection) => list_feed_id(&mut *connection, &message),
            Err(_error_message) => "error fetching data".to_string(),
        };

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

        let command = commands.replace(BOT_NAME, "");
        message.text = Some(command.clone());

        if command.starts_with("list_subscriptions") {
            let feed_id = Self::parse_int_from_string(&command);
            let feed_url = get_feed_url_by_id(db_pool, feed_id);
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params =
                set_list_subcriptions_menu_keyboard(message, feed_id.to_string(), feed_url);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("/list_subscriptions") {
            ListSubscriptions::execute(db_pool, api, message);
        } else if command.starts_with("/get_filter") {
            let feed_url =
                get_feed_url_by_id(db_pool.clone(), Self::parse_int_from_string(&command));
            message.text = Some(format!("/get_filter {}", feed_url));
            GetFilter::execute(db_pool, api, message);
        } else if command.starts_with("set_template") {
            let feed_id = Self::parse_int_from_string(&command);
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_template_menu_keyboard(message, feed_id.to_string());
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("substring") {
            api.delete_message(&delete_message_params).unwrap();
            let feed_id: i64 = Self::parse_int_from_string(&command);
            let data = command.replace("substring", "");
            let feed_url = get_feed_url_by_id(db_pool, feed_id);
            let send_message_params = set_template_substring_keyboard(message, data, feed_url);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("italic") {
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("italic", "");
            let send_message_params = set_template_italic_keyboard(message, data);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("bold") {
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("bold", "");
            let send_message_params = set_template_bold_keyboard(message, data);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("create_link") {
            api.delete_message(&delete_message_params).unwrap();
            let feed_id: i64 = Self::parse_int_from_string(&command);
            let data = command.replace("create_link", "");
            let feed_url = get_feed_url_by_id(db_pool, feed_id);
            let send_message_params = set_template_create_link_keyboard(message, data, feed_url);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("/set_template") {
            let feed_id = Self::parse_int_from_string(&command);
            let feed_url = get_feed_url_by_id(db_pool.clone(), feed_id);
            let text = command.replace(&feed_id.to_string(), &feed_url);
            message.text = Some(text.trim().to_string());
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with("set_default_template") {
            let feed_url =
                get_feed_url_by_id(db_pool.clone(), Self::parse_int_from_string(&command));
            message.text = Some(format!("/set_template {} {}", feed_url, DEFAULT_TEMPLATE));
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with("/get_template") {
            let feed_id = Self::parse_int_from_string(&command);
            println!("parsed feed id ======{}", feed_id);
            println!("feed id parsed using regex ==== {}", feed_id);
            let feed_url = get_feed_url_by_id(db_pool.clone(), feed_id);
            let text = command.replace(&feed_id.to_string(), &feed_url);
            message.text = Some(text.trim().to_string());
            GetTemplate::execute(db_pool, api, message);
        } else if command.starts_with("/remove_template") {
            let feed_id = Self::parse_int_from_string(&command);
            let feed_url = get_feed_url_by_id(db_pool.clone(), feed_id);
            let text = command.replace(&feed_id.to_string(), &feed_url);
            message.text = Some(text.trim().to_string());
            RemoveTemplate::execute(db_pool, api, message);
        } else if command.starts_with("/remove_filter") {
            let feed_id = Self::parse_int_from_string(&command);
            let feed_url = get_feed_url_by_id(db_pool.clone(), feed_id);
            let text = command.replace(&feed_id.to_string(), &feed_url);
            message.text = Some(text.trim().to_string());
            RemoveFilter::execute(db_pool, api, message);
        } else if command.starts_with("/set_global_template") {
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
        } else if command == "global_italic" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_italic_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "global_bold" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_bold_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "global_create_link" {
            let send_message_params = set_global_template_create_link_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("global_substring") {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_substring_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("/unsubscribe") {
            let feed_id = Self::parse_int_from_string(&command);
            let feed_url = get_feed_url_by_id(db_pool.clone(), feed_id);
            let text = command.replace(&feed_id.to_string(), &feed_url);
            message.text = Some(text.trim().to_string());
            Unsubscribe::execute(db_pool, api, message);
        } else if command.starts_with("unsubscribe") {
            let feed_id = Self::parse_int_from_string(&command);
            let feed_url = get_feed_url_by_id(db_pool.clone(), feed_id);
            message.text = Some(format!("/unsubscribe {}", feed_url));
            Unsubscribe::execute(db_pool, api, message);
        } else if command == "back to menu" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "Back to subscription list" {
            message.text = Some("/set_template".to_string());
            SetTemplate::execute(db_pool, api, message);
        } else {
            UnknownCommand::execute(db_pool, api, message)
        }

        fn list_feed_id(db_connection: &mut PgConnection, message: &Message) -> String {
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
                            .join("\n")
                    }
                }
            }
        }
    }
    fn parse_int_from_string(command: &str) -> i64 {
        let re = Regex::new(
            r"(?x)
            (?P<name>\d+)  # the name
        ",
        )
        .unwrap();
        let caps = re.captures(command).unwrap();
        let feed = caps["name"].trim().to_string();
        let feed_id: i64 = feed.parse().unwrap();
        feed_id
    }
}
pub fn get_feed_url_by_id(db_pool: Pool<ConnectionManager<PgConnection>>, data: i64) -> String {
    // println!("feed id from command replace {}", data);
    match fetch_db_connection(db_pool) {
        Ok(mut connection) => {
            let feeds = find(&mut *connection, data).unwrap();
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
