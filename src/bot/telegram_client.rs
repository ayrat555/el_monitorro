use crate::config::Config;
use fang::FangError;
use frankenstein::AllowedUpdate;
use frankenstein::ErrorResponse;
use frankenstein::GetUpdatesParams;
use frankenstein::ParseMode;
use frankenstein::SendMessageParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use isahc::{prelude::*, Request};
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Api {
    pub api_url: String,
    pub update_params: GetUpdatesParams,
    pub buffer: VecDeque<Update>,
}

#[derive(Debug)]
pub enum Error {
    HttpError(HttpError),
    ApiError(ErrorResponse),
}

#[derive(Eq, PartialEq, Debug)]
pub struct HttpError {
    pub code: u16,
    pub message: String,
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Error> for FangError {
    fn from(error: Error) -> Self {
        let description = format!("telegram error: {:?}", error);

        Self { description }
    }
}

impl Api {
    pub fn new() -> Api {
        let token = Config::telegram_bot_token();
        let base_url = Config::telegram_base_url();
        let api_url = format!("{}{}", base_url, token);

        let update_params = GetUpdatesParams::builder()
            .allowed_updates(vec![
                AllowedUpdate::Message,
                AllowedUpdate::ChannelPost,
                AllowedUpdate::CallbackQuery,
                AllowedUpdate::InlineQuery,
            ])
            .build();

        Api {
            api_url,
            update_params,
            buffer: VecDeque::new(),
        }
    }

    pub fn next_update(&mut self) -> Option<Update> {
        if let Some(update) = self.buffer.pop_front() {
            return Some(update);
        }

        match self.get_updates(&self.update_params) {
            Ok(updates) => {
                for update in updates.result {
                    self.buffer.push_back(update);
                }

                if let Some(last_update) = self.buffer.back() {
                    self.update_params.offset = Some((last_update.update_id + 1).into());
                }

                self.buffer.pop_front()
            }

            Err(err) => {
                log::error!("Failed to fetch updates {:?}", err);
                None
            }
        }
    }

    pub fn send_text_message(&self, chat_id: i64, message: String) -> Result<(), Error> {
        self.reply_with_text_message(chat_id, message, None)
    }

    pub fn reply_with_text_message(
        &self,
        chat_id: i64,
        message: String,
        message_id: Option<i32>,
    ) -> Result<(), Error> {
        let send_message_params = match message_id {
            None => SendMessageParams::builder()
                .chat_id(chat_id)
                .text(message)
                .parse_mode(ParseMode::Html)
                .build(),

            Some(message_id_value) => SendMessageParams::builder()
                .chat_id(chat_id)
                .text(message)
                .parse_mode(ParseMode::Html)
                .reply_to_message_id(message_id_value)
                .build(),
        };

        match self.send_message(&send_message_params) {
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

impl TelegramApi for Api {
    type Error = Error;

    fn request<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, Error> {
        let url = format!("{}/{}", self.api_url, method);

        let request_builder = Request::post(url).header("Content-Type", "application/json");

        let mut response = match params {
            None => request_builder.body(())?.send()?,
            Some(data) => {
                let json = serde_json::to_string(&data).unwrap();
                request_builder.body(json)?.send()?
            }
        };

        let mut bytes = Vec::new();
        response.copy_to(&mut bytes)?;

        let parsed_result: Result<T2, serde_json::Error> = serde_json::from_slice(&bytes);

        match parsed_result {
            Ok(result) => Ok(result),
            Err(_) => {
                let parsed_error: Result<ErrorResponse, serde_json::Error> =
                    serde_json::from_slice(&bytes);

                match parsed_error {
                    Ok(result) => Err(Error::ApiError(result)),
                    Err(error) => {
                        let message = format!("{:?} {:?}", error, std::str::from_utf8(&bytes));

                        let error = HttpError { code: 500, message };

                        Err(Error::HttpError(error))
                    }
                }
            }
        }
    }

    // isahc doesn't support multipart uploads
    // https://github.com/sagebind/isahc/issues/14
    // but it's fine because this bot doesn't need this feature
    fn request_with_form_data<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        _method: &str,
        _params: T1,
        _files: Vec<(&str, PathBuf)>,
    ) -> Result<T2, Error> {
        let error = HttpError {
            code: 500,
            message: "isahc doesn't support form data requests".to_string(),
        };

        Err(Error::HttpError(error))
    }
}
