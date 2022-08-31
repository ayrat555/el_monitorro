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
use crate::bot::telegram_client::Api;
use crate::config::Config;
use diesel::r2d2;
use diesel::PgConnection;
use frankenstein::Update;
use frankenstein::UpdateContent;
use std::thread;

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

        let command = &text.unwrap();

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
}
