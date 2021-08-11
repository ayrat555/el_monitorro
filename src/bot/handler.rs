use super::commands::help::Help;
use super::commands::subscribe::Subscribe;
use crate::bot::telegram_client::Api;
use crate::bot::telegram_client::Error;
use frankenstein::Update;
use std::env;

struct Handler {
    api: Api,
}

static COMMANDS: [&str; 2] = [Help::command(), Subscribe::command()];

impl Handler {
    pub async fn start() {
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

        let mut api = Api::new(token);
        // let mut stream = api.stream();

        log::info!("Starting a bot");

        let mut interval = time::interval(std::time::Duration::from_secs(1));

        loop {
            while let Some(update) = api.next_update() {
                tokio::spawn(self.process_message_or_channel_post(api.clone(), update));
            }

            interval.tick().await;
        }
    }

    fn process_message_or_channel_post(&self, api: Api, update: Update) {
        let message = match update.message() {
            None => update.channel_post().unwrap(),
            Some(message) => message,
        };

        let chat_id = message.chat().id() as i64;

        if let Some(id) = self.owner_telegram_id() {
            if id != chat_id {
                return;
            }
        }

        let text = message.text();

        if text.is_none() {
            return;
        }

        let command = &text.unwrap();

        let is_known_command = COMMANDS
            .iter()
            .any(|command_name| command.starts_with(command_name));

        if is_known_command {
            log::info!("{:?} wrote: {}", chat_id, command);
        }
    }

    fn owner_telegram_id(&self) -> Option<i64> {
        match env::var("OWNER_TELEGRAM_ID") {
            Ok(val) => {
                let parsed_value: i64 = val.parse().unwrap();
                Some(parsed_value)
            }
            Err(_error) => None,
        }
    }
}
