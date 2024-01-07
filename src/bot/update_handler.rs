use super::commands::CommandProcessor;
use crate::bot::telegram_client;
use crate::config::Config;
use frankenstein::MaybeInaccessibleMessage;
use frankenstein::Update;
use frankenstein::UpdateContent;
use std::thread;

pub struct UpdateHandler {}

impl UpdateHandler {
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
                thread_pool.spawn(move || match update.content {
                    UpdateContent::CallbackQuery(_) => {
                        Self::process_callback_query(update);
                    }
                    UpdateContent::Message(_) | UpdateContent::ChannelPost(_) => {
                        Self::process_message_or_channel_post(update);
                    }
                    _ => (),
                });
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

        CommandProcessor::builder()
            .message(message)
            .text(text.unwrap())
            .callback(false)
            .build()
            .process()
    }

    fn process_callback_query(update: Update) {
        let query = match update.content {
            UpdateContent::CallbackQuery(callback_query) => callback_query,
            _ => return,
        };

        let text = query.data.clone();

        if text.is_none() || query.message.is_none() {
            return;
        }

        if let MaybeInaccessibleMessage::Message(message) = query.message.unwrap() {
            CommandProcessor::builder()
                .message(message)
                .text(text.unwrap())
                .callback(true)
                .build()
                .process();
        }
    }

    fn owner_telegram_id() -> Option<i64> {
        Config::owner_telegram_id()
    }
}
