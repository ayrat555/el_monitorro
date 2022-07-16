use crate::config::Config;
use frankenstein::AllowedUpdate;
use frankenstein::AsyncApi;
use frankenstein::AsyncTelegramApi;
use frankenstein::Error as ApiErr;
use frankenstein::GetUpdatesParams;
use frankenstein::ParseMode;
use frankenstein::SendMessageParams;
use frankenstein::Update;
use isahc::{prelude::*, Request};
use std::collections::VecDeque;
use std::path::PathBuf;

static BASE_API_URL: &str = "https://api.telegram.org/bot";

#[derive(Clone)]
pub struct Api {
    pub async_api: AsyncApi,
    pub update_params: GetUpdatesParams,
    pub buffer: VecDeque<Update>,
}

#[derive(Debug)]
pub enum Error {
    HttpError(HttpError),
    ApiError(ApiErr),
}

#[derive(PartialEq, Debug)]
pub struct HttpError {
    pub code: u16,
    pub message: String,
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

impl Api {
    pub fn new() -> Api {
        let token = Config::telegram_bot_token();
        let api = AsyncApi::new(&token);

        let update_params = GetUpdatesParams::builder()
            .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::ChannelPost])
            .build();

        Api {
            async_api: api,
            update_params,
            buffer: VecDeque::new(),
        }
    }

    pub async fn next_update(&mut self) -> Option<Update> {
        if let Some(update) = self.buffer.pop_front() {
            return Some(update);
        }

        match self.async_api.get_updates(&self.update_params).await {
            Ok(updates) => {
                for update in updates.result {
                    self.buffer.push_back(update);
                }

                if let Some(last_update) = self.buffer.back() {
                    self.update_params.offset = Some(last_update.update_id + 1);
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
        let send_message_params = SendMessageParams::builder()
            .chat_id(chat_id)
            .parse_mode(ParseMode::Html)
            .text(message)
            .build();

        match self.async_api.send_message(&send_message_params).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(
                    "Failed to send message {:?}: {:?}",
                    err, send_message_params
                );
                Err(Error::ApiError(err))
            }
        }
    }
}

impl From<isahc::http::Error> for Error {
    fn from(error: isahc::http::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError { code: 500, message };

        Error::HttpError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError { code: 500, message };

        Error::HttpError(error)
    }
}

impl From<isahc::Error> for Error {
    fn from(error: isahc::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError { code: 500, message };

        Error::HttpError(error)
    }
}
