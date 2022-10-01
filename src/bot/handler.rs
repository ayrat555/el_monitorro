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
use super::commands::BotCommand;
use super::commands::GetFilter;
use crate::bot::telegram_client;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use diesel::r2d2;
use diesel::PgConnection;
use frankenstein::Update;
use frankenstein::UpdateContent;
use std::str::FromStr;
use std::thread;

pub struct Handler {}

impl Handler {
    pub fn start() {
        let mut api = telegram_client::api().clone();
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

                thread_pool
                    .spawn(move || Self::process_message_or_channel_post(db_pool, tg_api, update));
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

        let command = BotCommand::from_str(&text.unwrap()).unwrap();

        match command {
            BotCommand::Subscribe(args) => Subscribe::execute(db_pool, api, message, args),

            BotCommand::Help => Help::execute(db_pool, api, message),

            BotCommand::Unsubscribe(args) => Unsubscribe::execute(db_pool, api, message, args),

            BotCommand::ListSubscriptions => ListSubscriptions::execute(db_pool, api, message),

            BotCommand::Start => Start::execute(db_pool, api, message),

            BotCommand::SetTimezone(args) => SetTimezone::execute(db_pool, api, message, args),

            BotCommand::GetTimezone => GetTimezone::execute(db_pool, api, message, args),

            BotCommand::SetFilter(args) => SetFilter::execute(db_pool, api, message, args),

            BotCommand::GetFilter(args) => {
                let get_filter = GetFilter::builder()
                    .db_pool(db_pool)
                    .api(api)
                    .message(message)
                    .args(args)
                    .build();

                get_filter.run();
            }
            BotCommand::RemoveFilter(args) => RemoveFilter::execute(db_pool, api, message, args),

            BotCommand::SetTemplate(args) => SetTemplate::execute(db_pool, api, message, args),

            BotCommand::GetTemplate(args) => GetTemplate::execute(db_pool, api, message, args),

            BotCommand::RemoveTemplate(args) => {
                RemoveTemplate::execute(db_pool, api, message, args)
            }
            BotCommand::SetGlobalTemplate(args) => {
                SetGlobalTemplate::execute(db_pool, api, message, args)
            }

            BotCommand::RemoveGlobalTemplate => {
                RemoveGlobalTemplate::execute(db_pool, api, message)
            }

            BotCommand::GetGlobalTemplate => GetGlobalTemplate::execute(db_pool, api, message),

            BotCommand::SetGlobalFilter(args) => {
                SetGlobalFilter::execute(db_pool, api, message, args)
            }

            BotCommand::GetGlobalFilter => {
                let get_filter = GetGlobalFilter::builder()
                    .db_pool(db_pool)
                    .api(api)
                    .message(message)
                    .build();

                get_filter.run();
            }

            BotCommand::RemoveGlobalFilter => RemoveGlobalFilter::execute(db_pool, api, message),

            BotCommand::Info => Info::execute(db_pool, api, message),

            BotCommand::SetContentFields(args) => {
                SetContentFields::execute(db_pool, api, message, args)
            }

            BotCommand::UnknownCommand(args) => {
                UnknownCommand::execute(db_pool, api, message, args)
            }
        };
    }

    fn owner_telegram_id() -> Option<i64> {
        Config::owner_telegram_id()
    }
}
