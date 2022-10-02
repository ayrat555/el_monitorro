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
use crate::config::Config;
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
                thread_pool.spawn(move || Self::process_message_or_channel_post(update));
            }

            thread::sleep(interval);
        }
    }

    fn process_message_or_channel_post(update: Update) {
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
            BotCommand::Subscribe(args) => Subscribe::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::Help => Help::builder().message(message).build().run(),

            BotCommand::Unsubscribe(args) => Unsubscribe::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::ListSubscriptions => {
                ListSubscriptions::builder().message(message).build().run()
            }

            BotCommand::Start => Start::builder().message(message).build().run(),

            BotCommand::SetTimezone(args) => SetTimezone::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::GetTimezone => GetTimezone::builder().message(message).build().run(),

            BotCommand::SetFilter(args) => SetFilter::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::GetFilter(args) => GetFilter::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::RemoveFilter(args) => RemoveFilter::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::SetTemplate(args) => SetTemplate::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::GetTemplate(args) => GetTemplate::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::RemoveTemplate(args) => RemoveTemplate::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::SetGlobalTemplate(args) => SetGlobalTemplate::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::RemoveGlobalTemplate => RemoveGlobalTemplate::builder()
                .message(message)
                .build()
                .run(),

            BotCommand::GetGlobalTemplate => {
                GetGlobalTemplate::builder().message(message).build().run()
            }

            BotCommand::SetGlobalFilter(args) => SetGlobalFilter::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::GetGlobalFilter => {
                GetGlobalFilter::builder().message(message).build().run()
            }

            BotCommand::RemoveGlobalFilter => {
                RemoveGlobalFilter::builder().message(message).build().run()
            }

            BotCommand::Info => Info::builder().message(message).build().run(),

            BotCommand::SetContentFields(args) => SetContentFields::builder()
                .message(message)
                .args(args)
                .build()
                .run(),

            BotCommand::UnknownCommand(args) => UnknownCommand::builder()
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
