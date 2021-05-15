use frankenstein::ErrorResponse;
use frankenstein::GetUpdatesParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use isahc::{prelude::*, Request};
use std::collections::VecDeque;
use std::path::PathBuf;

static BASE_API_URL: &str = "https://api.telegram.org/bot";

#[derive(PartialEq, Clone, Debug)]
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

#[derive(PartialEq, Debug)]
pub struct HttpError {
    pub code: u16,
    pub message: String,
}

impl Api {
    pub fn new(api_key: String) -> Api {
        let api_url = format!("{}{}", BASE_API_URL, api_key);

        let mut update_params = GetUpdatesParams::new();
        update_params.set_allowed_updates(Some(vec![
            "message".to_string(),
            "channel_post".to_string(),
        ]));

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
                    self.update_params
                        .set_offset(Some(last_update.update_id() + 1));
                }

                self.buffer.pop_front()
            }

            Err(err) => {
                log::error!("Failed to fetch updates {:?}", err);
                None
            }
        }
    }
}

impl From<isahc::http::Error> for Error {
    fn from(error: isahc::http::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError {
            code: 500,
            message,
        };

        Error::HttpError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError {
            code: 500,
            message,
        };

        Error::HttpError(error)
    }
}

impl From<isahc::Error> for Error {
    fn from(error: isahc::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError {
            code: 500,
            message,
        };

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

        let text = response.text()?;

        let parsed_result: Result<T2, serde_json::Error> = serde_json::from_str(&text);

        match parsed_result {
            Ok(result) => Ok(result),
            Err(_) => {
                let parsed_error: Result<ErrorResponse, serde_json::Error> =
                    serde_json::from_str(&text);

                match parsed_error {
                    Ok(result) => Err(Error::ApiError(result)),
                    Err(error) => {
                        let message = format!("{:?}", error);

                        let error = HttpError {
                            code: 500,
                            message,
                        };

                        Err(Error::HttpError(error))
                    }
                }
            }
        }
    }

    // isahc doesn't support multipart uploads
    // https://github.com/sagebind/isahc/issues/14
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
