use crate::config::Config;
use frankenstein::AsyncApi;
use frankenstein::AsyncTelegramApi;
use frankenstein::Error;
use frankenstein::GetUpdatesParamsBuilder;
use frankenstein::SendMessageParamsBuilder;
use frankenstein::Update;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Api {
    pub api: AsyncApi,
    pub update_params_builder: GetUpdatesParamsBuilder,
    pub buffer: VecDeque<Update>,
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

impl Api {
    pub fn new() -> Self {
        let token = Config::telegram_bot_token();
        let api = AsyncApi::new(&token);

        let mut update_params_builder = GetUpdatesParamsBuilder::default();
        update_params_builder
            .allowed_updates(vec!["message".to_string(), "channel_post".to_string()]);

        Api {
            api,
            update_params_builder,
            buffer: VecDeque::new(),
        }
    }

    pub async fn next_update(&mut self) -> Option<Update> {
        if let Some(update) = self.buffer.pop_front() {
            return Some(update);
        }

        let update_params = self.update_params_builder.build().unwrap();

        match self.api.get_updates(&update_params).await {
            Ok(updates) => {
                for update in updates.result {
                    self.buffer.push_back(update);
                }

                if let Some(last_update) = self.buffer.back() {
                    self.update_params_builder.offset(last_update.update_id + 1);
                }

                self.buffer.pop_front()
            }

            Err(err) => {
                log::error!("Failed to fetch updates {:?}", err);
                None
            }
        }
    }

    pub async fn send_text_message(&self, chat_id: i64, message: String) -> Result<(), Error> {
        let send_message_params = SendMessageParamsBuilder::default()
            .chat_id(chat_id)
            .text(message)
            .build()
            .unwrap();

        match self.api.send_message(&send_message_params).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(
                    "Failed to send message {:?}: {:?}",
                    err, send_message_params
                );
                Err(err)
            }
        }
    }
}
