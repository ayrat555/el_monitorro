use crate::config::Config;
use crate::http_client;
use fang::FangError;
use frankenstein::AllowedUpdate;
use frankenstein::DeleteMessageParams;
use frankenstein::ErrorResponse;
use frankenstein::GetUpdatesParams;
use frankenstein::LinkPreviewOptions;
use frankenstein::Message;
use frankenstein::ParseMode;
use frankenstein::ReplyParameters;
use frankenstein::SendMessageParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use isahc::prelude::*;
use isahc::HttpClient;
use isahc::Request;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::OnceLock;
use typed_builder::TypedBuilder;

static API: OnceLock<Api> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Api {
    pub api_url: String,
    pub update_params: GetUpdatesParams,
    pub buffer: VecDeque<Update>,
    pub http_client: HttpClient,
}

#[derive(Debug)]
pub enum Error {
    Http { code: u16, message: String },
    Api(ErrorResponse),
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Error> for FangError {
    fn from(error: Error) -> Self {
        let description = format!("telegram error: {error:?}");

        Self { description }
    }
}

#[derive(TypedBuilder)]
pub struct SimpleMessageParams {
    chat_id: i64,
    message: String,
    #[builder(setter(into), default)]
    reply_message_id: Option<i32>,
    #[builder(default = false)]
    preview_enabled: bool,
    #[builder(default, setter(into))]
    message_thread_id: Option<i32>,
}

impl Api {
    pub fn new() -> Api {
        let token = Config::telegram_bot_token();
        let base_url = Config::telegram_base_url();
        let api_url = format!("{base_url}{token}");
        let http_client = http_client::client().clone();

        let update_params = GetUpdatesParams::builder()
            .allowed_updates(vec![
                AllowedUpdate::Message,
                AllowedUpdate::ChannelPost,
                AllowedUpdate::CallbackQuery,
            ])
            .build();

        Api {
            api_url,
            update_params,
            http_client,
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

    pub fn reply_with_text_message(
        &self,
        simple_params: &SimpleMessageParams,
    ) -> Result<(), Error> {
        let preview_params = LinkPreviewOptions::builder()
            .is_disabled(!simple_params.preview_enabled)
            .build();

        let mut message_params = SendMessageParams::builder()
            .chat_id(simple_params.chat_id)
            .text(simple_params.message.clone())
            .link_preview_options(preview_params)
            .parse_mode(ParseMode::Html)
            .build();

        message_params.message_thread_id = simple_params.message_thread_id;

        if let Some(message_id) = simple_params.reply_message_id {
            let reply_params = ReplyParameters::builder().message_id(message_id).build();

            message_params.reply_parameters = Some(reply_params);
        }

        self.send_message_with_params(&message_params)
    }

    pub fn send_message_with_params(
        &self,
        send_message_params: &SendMessageParams,
    ) -> Result<(), Error> {
        match self.send_message(send_message_params) {
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

    pub fn remove_message(&self, message: &Message) {
        let params = DeleteMessageParams::builder()
            .chat_id(message.chat.id)
            .message_id(message.message_id)
            .build();

        if let Err(err) = self.delete_message(&params) {
            error!("Failed to delete a message {:?}: {:?}", err, params);
        }
    }
}

impl From<isahc::http::Error> for Error {
    fn from(error: isahc::http::Error) -> Self {
        let message = format!("{error:?}");

        Self::Http { code: 500, message }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let message = format!("{error:?}");

        Self::Http { code: 500, message }
    }
}

impl From<isahc::Error> for Error {
    fn from(error: isahc::Error) -> Self {
        let message = format!("{error:?}");

        Self::Http { code: 500, message }
    }
}

impl TelegramApi for Api {
    type Error = Error;

    fn request<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, Error> {
        let url = format!("{}/{method}", self.api_url);

        let request_builder = Request::post(url).header("Content-Type", "application/json");

        let mut response = match params {
            None => {
                let request = request_builder.body(())?;
                self.http_client.send(request)?
            }
            Some(data) => {
                let json = serde_json::to_string(&data).unwrap();
                let request = request_builder.body(json)?;

                self.http_client.send(request)?
            }
        };

        let mut bytes = Vec::new();
        response.copy_to(&mut bytes)?;

        serde_json::from_slice(&bytes).map_err(|_| {
            match serde_json::from_slice::<ErrorResponse>(&bytes) {
                Ok(result) => {
                    log::error!(
                        "Failed to parse a  response {result:?} - {:?}",
                        std::str::from_utf8(&bytes)
                    );
                    Error::Api(result)
                }
                Err(error) => {
                    log::error!(
                        "Failed to parse an error response {error:?} - {:?}",
                        std::str::from_utf8(&bytes)
                    );

                    Error::Http {
                        code: 500,
                        message: format!("{error:?}"),
                    }
                }
            }
        })
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
        let message = "isahc doesn't support form data requests".to_string();
        Err(Error::Http { code: 500, message })
    }
}

pub fn api() -> &'static Api {
    API.get_or_init(Api::new)
}
