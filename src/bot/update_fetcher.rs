use frankenstein::Api;
use frankenstein::GetUpdatesParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use std::collections::VecDeque;

#[derive(PartialEq, Debug, Clone)]
pub struct UpdateFetcher {
    pub api: Api,
    pub update_params: GetUpdatesParams,
    pub buffer: VecDeque<Update>,
}

impl UpdateFetcher {
    pub fn new(api: Api) -> UpdateFetcher {
        let mut update_params = GetUpdatesParams::new();
        update_params.set_allowed_updates(Some(vec![
            "message".to_string(),
            "channel_post".to_string(),
        ]));

        UpdateFetcher {
            api,
            update_params,
            buffer: VecDeque::new(),
        }
    }

    pub fn next_update(&mut self) -> Option<Update> {
        if let Some(update) = self.buffer.pop_front() {
            return Some(update);
        }

        match self.api.get_updates(&self.update_params) {
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
