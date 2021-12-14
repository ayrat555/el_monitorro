use super::commands::get_filter::GetFilter;
use super::commands::get_global_template::GetGlobalTemplate;
use super::commands::get_template::GetTemplate;
use super::commands::get_timezone::GetTimezone;
use super::commands::help::Help;
use super::commands::info::Info;
use super::commands::list_subscriptions::ListSubscriptions;
use super::commands::set_filter::SetFilter;
use super::commands::set_global_template::SetGlobalTemplate;
use super::commands::set_template::SetTemplate;
use super::commands::set_timezone::SetTimezone;
use super::commands::start::Start;
use super::commands::subscribe::Subscribe;
use super::commands::unknown_command::UnknownCommand;
use super::commands::unsubscribe::Unsubscribe;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db;
use diesel::r2d2;
use diesel::PgConnection;
use frankenstein::Update;
use tokio::time;

pub struct Handler {}

impl Handler {
    pub async fn start() {
        let mut api = Api::default();
        let connection_pool = db::create_connection_pool(Config::commands_db_pool_number());

        log::info!("Starting the El Monitorro bot");

        let mut interval = time::interval(std::time::Duration::from_secs(1));

        loop {
            while let Some(update) = api.next_update() {
                tokio::spawn(Self::process_message_or_channel_post(
                    connection_pool.clone(),
                    api.clone(),
                    update,
                ));
            }

            interval.tick().await;
        }
    }

    async fn process_message_or_channel_post(
        db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
        api: Api,
        update: Update,
    ) {
        let message = match update.message {
            None => update.channel_post.unwrap(),
            Some(message) => message,
        };

        let chat_id = message.chat.id as i64;

        if let Some(id) = Self::owner_telegram_id() {
            if id != chat_id {
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
        } else if command.starts_with(SetTemplate::command()) {
            SetTemplate::execute(db_pool, api, message);
        } else if command.starts_with(GetTemplate::command()) {
            GetTemplate::execute(db_pool, api, message);
        } else if command.starts_with(SetGlobalTemplate::command()) {
            SetGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with(GetGlobalTemplate::command()) {
            GetGlobalTemplate::execute(db_pool, api, message);
        } else if command.starts_with(Info::command()) {
            Info::execute(db_pool, api, message);
        } else {
            UnknownCommand::execute(db_pool, api, message);
        }
    }

    fn owner_telegram_id() -> Option<i64> {
        Config::owner_telegram_id()
    }
}
