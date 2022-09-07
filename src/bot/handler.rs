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

const BOT_NAME: &str = "@el_monitorro_bot "; //replace it with your botname,this const is used to remove bot name from the command

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

<<<<<<< HEAD
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
=======
        let commands = &text.unwrap();
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
>>>>>>> 64539e0 (added inline keyboard for setting global template)
    }

    fn owner_telegram_id() -> Option<i64> {
        Config::owner_telegram_id()
    }
}
