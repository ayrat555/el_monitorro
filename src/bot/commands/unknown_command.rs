use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use frankenstein::ChatType;
use typed_builder::TypedBuilder;

static UNKNOWN_COMMAND_GROUP: &str = "Remove admin access from the bot in this group otherwise it will be replying to every message.";
static UNKNOWN_COMMAND_PRIVATE: &str = "Unknown command. Use /help to show available commands";

static COMMAND: &str = "";

#[derive(TypedBuilder)]
pub struct UnknownCommand {
    api: Api,
    message: Message,
    args: String,
}

impl UnknownCommand {
    pub fn run(&self) {
        self.execute(&self.api, &self.message);
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for UnknownCommand {
    fn response(&self) -> String {
        match self.message.chat.type_field {
            ChatType::Private => UNKNOWN_COMMAND_PRIVATE.to_string(),
            ChatType::Group | ChatType::Supergroup => {
                if self.message.text.as_ref().unwrap().starts_with('/')
                    || self.message.reply_to_message.is_some()
                {
                    "".to_string()
                } else {
                    UNKNOWN_COMMAND_GROUP.to_string()
                }
            }
            ChatType::Channel => "".to_string(),
        }
    }

    fn execute(&self, api: &Api, message: &Message) {
        if message.chat.type_field != ChatType::Channel {
            info!("{:?} wrote: {}", message.chat.id, self.args);
        }

        let text = self.response();

        if !text.is_empty() {
            self.reply_to_message(api, message, text);
        }
    }
}
