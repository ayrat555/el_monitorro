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
use crate::bot::commands::set_global_template::set_global_template_bold_keyboard;
use crate::bot::commands::set_global_template::set_global_template_create_link_keyboard;
use crate::bot::commands::set_global_template::set_global_template_italic_keyboard;
use crate::bot::commands::set_global_template::set_global_template_keyboard;
use crate::bot::commands::set_global_template::set_global_template_substring_keyboard;
use crate::bot::commands::set_template::select_feed_url;
use crate::bot::commands::set_template::set_template_bold_keyboard;
use crate::bot::commands::set_template::set_template_italic_keyboard;
use crate::bot::commands::set_template::set_template_keyboard;
use crate::bot::commands::set_template::set_template_menu_keyboard;
use crate::bot::commands::unsubscribe::select_feed_url_unsubscribe;
use crate::bot::commands::unsubscribe::set_unsubscribe_keyboard;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use diesel::r2d2;
use diesel::PgConnection;
use frankenstein::DeleteMessageParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use frankenstein::UpdateContent;
use std::thread;

const BOT_NAME: &str = "@sasaathulbot "; //replace it with your botname,this const is used to remove bot name from the command

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
                //   println!("updat.content  ========{:?}",update.content.clone());
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
        // let data =update.clone();

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

        // let data =query.data.clone();
        if text.is_none() {
            return;
        }

        let commands = &text.unwrap();
        let delete_message_params = DeleteMessageParams::builder()
            .chat_id(message.chat.id)
            .message_id(message.message_id)
            .build();
        // let query_data =&data.unwrap();
        // println!("query data = {}",query_data);
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
        let query = match update.content.clone() {
            UpdateContent::CallbackQuery(callback_query) => callback_query,
            _ => return,
        };

        let mut message = query.message.unwrap();
        let messageid = message.message_id;
        let chatid = message.chat.id;
        println!("before updating text ={:?}", message.text);
        let text = query.data.clone();
        let delete_message_params = DeleteMessageParams::builder()
            .chat_id(chatid)
            .message_id(messageid)
            .build();
        if text.is_none() {
            return;
        }

        let commands = &text.unwrap();

        println!("command = {}", commands);
        let mut command = commands.replace(BOT_NAME, "");

        //removes bot name from the command (switch_inline_query_current_chat adds botname automatically)
        message.text = Some(command.clone());
        println!("after updating text ={:?}", message.text);

        if command == "/set_global_template {{italic bot_item_description }}" {
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command == "italic" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_italic_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("/unsubscribe") {
            message.text = Some(command);
            Unsubscribe::execute(db_pool, api, message);
        } else if command == "bold" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_template_bold_keyboard(message, command);
            api.send_message(&send_message_params).unwrap();
        } else if command == "/set_global_template {{italic bot_item_name }}" {
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command == "create_link" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_create_link_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "/set_global_template {{create_link bot_item_description}}" {
            message.text = Some(
                "/set_global_template {{create_link bot_item_description bot_item_link}}"
                    .to_string(),
            );
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command == "/set_global_template {{create_link bot_item_name}}" {
            message.text = Some(
                "/set_global_template {{create_link bot_item_name bot_item_link}}".to_string(),
            );
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with("bold_set_template") {
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("bold_set_template", "");
            let send_message_params = set_template_bold_keyboard(message, data);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("/set_template_bold_des") {
            //handling callbackquery of setting set template feed url bold
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("/set_template_bold_des", "");
            message.text = Some(format!(
                "/set_template{} {{bold bot_item_description}}",
                data
            ));
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with("/set_template_bold_item") {
            //handling callbackquery of setting set template feed url bold
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("/set_template_bold_item", "");
            message.text = Some(format!("/set_template{} {{bold bot_item_name}}", data));
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with("italic_set_template") {
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("italic_set_template", "");
            let send_message_params = set_template_italic_keyboard(message, data);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("/set_template_italic_des") {
            //handling callbackquery of setting set template feed url bold
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("/set_template_italic_des", "");
            message.text = Some(format!(
                "/set_template{} {{bold bot_item_description}}",
                data
            ));
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with("/set_template_italic_item") {
            //handling callbackquery of setting set template feed url bold
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("/set_template_italic_item", "");
            message.text = Some(format!("/set_template{} {{italic bot_item_name}}", data));
            SetTemplate::execute(db_pool, api, message);
        } else if command == "/set_global_template {{bold bot_item_name }}" {
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command == "/set_global_template {{bold bot_item_description }}" {
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with("substring") {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_substring_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "back to menu" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command == "back to set_template menu" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_template_menu_keyboard(message, command);
            api.send_message(&send_message_params).unwrap();
        } else if command == "/set_template {{bot_item_description}}" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if command.starts_with("feed_for_template") {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_template_menu_keyboard(message.clone(), command);
            api.send_message(&send_message_params).unwrap();
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with("/set_template_des") {
            api.delete_message(&delete_message_params).unwrap();
            let data = command.replace("/set_template_des", "");
            message.text = Some(format!(
                "/set_template{} {{create_link bot_item_description bot_item_link}}",
                data
            ));
            SetTemplate::execute(db_pool, api, message);
        } else {
            // UnknownCommand::execute(db_pool, api, message);
            println!("no command incoming")
        }
    }
}
