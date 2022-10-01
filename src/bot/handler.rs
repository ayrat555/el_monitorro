use super::commands::BotCommand;
use super::commands::GetFilter;
use super::commands::GetGlobalFilter;
use super::commands::GetGlobalTemplate;
use super::commands::GetTemplate;
use super::commands::GetTimezone;
use super::commands::Help;
use super::commands::Info;
use super::commands::ListSubscriptions;
use super::commands::RemoveFilter;
use super::commands::RemoveGlobalFilter;
use super::commands::RemoveGlobalTemplate;
use super::commands::RemoveTemplate;
use super::commands::SetContentFields;
use super::commands::SetFilter;
use super::commands::SetGlobalFilter;
use super::commands::SetGlobalTemplate;
use super::commands::SetTemplate;
use super::commands::SetTimezone;
use super::commands::Start;
use super::commands::Subscribe;
use super::commands::UnknownCommand;
use super::commands::Unsubscribe;
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

            BotCommand::Help => {
                let command = Help::builder().api(api).message(message).build();

                command.run();
            }

            BotCommand::Unsubscribe(args) => Unsubscribe::execute(db_pool, api, message, args),

            BotCommand::ListSubscriptions => ListSubscriptions::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .build()
                .run(),

            BotCommand::Start => Start::execute(db_pool, api, message),

            BotCommand::SetTimezone(args) => SetTimezone::execute(db_pool, api, message, args),

            BotCommand::GetTimezone => GetTimezone::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .build()
                .run(),

            BotCommand::SetFilter(args) => SetFilter::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::GetFilter(args) => {
                GetFilter::builder()
                    .db_pool(db_pool)
                    .api(api)
                    .message(message)
                    .args(args)
                    .build()
                    .run();
            }

            BotCommand::RemoveFilter(args) => RemoveFilter::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::SetTemplate(args) => SetTemplate::execute(db_pool, api, message, args),

            BotCommand::GetTemplate(args) => GetTemplate::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::RemoveTemplate(args) => RemoveTemplate::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::SetGlobalTemplate(args) => SetGlobalTemplate::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::RemoveGlobalTemplate => RemoveGlobalTemplate::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .build()
                .run(),

            BotCommand::GetGlobalTemplate => GetGlobalTemplate::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .build()
                .run(),

            BotCommand::SetGlobalFilter(args) => SetGlobalFilter::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::GetGlobalFilter => GetGlobalFilter::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .build()
                .run(),

            BotCommand::RemoveGlobalFilter => RemoveGlobalFilter::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .build()
                .run(),

            BotCommand::Info => Info::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .build()
                .run(),

            BotCommand::SetContentFields(args) => SetContentFields::builder()
                .db_pool(db_pool)
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::UnknownCommand(args) => UnknownCommand::builder()
                .api(api)
                .message(message)
                .args(args)
                .build()
                .run(),
        };
    }

    fn owner_telegram_id() -> Option<i64> {
        Config::owner_telegram_id()
    }
}
