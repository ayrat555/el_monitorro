use super::Close;
use super::Command;
use super::Help;
use super::Message;
use super::Response;
use frankenstein::ChatType;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::LinkPreviewOptions;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static UNKNOWN_COMMAND_GROUP: &str = "Remove admin access from the bot in this group otherwise it will be replying to every message.";
static UNKNOWN_COMMAND_PRIVATE: &str = "Unknown command";

static COMMAND: &str = "";

#[derive(TypedBuilder)]
pub struct UnknownCommand {
    message: Message,
    args: String,
}

impl UnknownCommand {
    pub fn run(&self) {
        self.execute(&self.message, &format!("unknown command {}", self.args));
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for UnknownCommand {
    fn execute(&self, message: &Message, command: &str) {
        if message.chat.type_field != ChatType::Channel {
            info!("{:?} wrote: {}", message.chat.id, command);
        }

        let response = match self.message.chat.type_field {
            ChatType::Private => Some(UNKNOWN_COMMAND_PRIVATE.to_string()),
            ChatType::Group | ChatType::Supergroup => {
                if self.message.text.as_ref().unwrap().starts_with('/')
                    || self.message.reply_to_message.is_some()
                {
                    None
                } else {
                    Some(UNKNOWN_COMMAND_GROUP.to_string())
                }
            }
            ChatType::Channel => None,
        };

        if let Some(text) = response {
            let buttons: Vec<Vec<InlineKeyboardButton>> =
                vec![Help::button_row(), Close::button_row()];

            let keyboard = InlineKeyboardMarkup::builder()
                .inline_keyboard(buttons)
                .build();

            let preview_params = LinkPreviewOptions::builder().is_disabled(true).build();

            let mut params = SendMessageParams::builder()
                .chat_id(self.message.chat.id)
                .text(text)
                .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
                .link_preview_options(preview_params)
                .build();

            params.message_thread_id = message.message_thread_id;

            self.send_message(params)
        }
    }

    // placeholder, not used
    fn response(&self) -> Response {
        Response::Simple("".to_string())
    }
}
