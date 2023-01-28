use super::Close;
use super::Command;
use super::GetFilter;
use super::GetPreviewEnabled;
use super::GetTemplate;
use super::RemoveFilter;
use super::RemoveTemplate;
use super::Response;
use super::TogglePreviewEnabled;
use super::Unsubscribe;
use diesel::PgConnection;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/feed_keyboard";

#[derive(TypedBuilder)]
pub struct ShowFeedKeyboard {
    message: Message,
    feed_url: String,
}

impl ShowFeedKeyboard {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    fn feed_keyboard(&self, db_connection: &mut PgConnection) -> SendMessageParams {
        if let Err(message) =
            self.find_subscription(db_connection, self.message.chat.id, &self.feed_url)
        {
            return SendMessageParams::builder()
                .chat_id(self.message.chat.id)
                .text(message)
                .build();
        };

        let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let rows = [
            vec![GetFilter::command(), RemoveFilter::command()],
            vec![GetTemplate::command(), RemoveTemplate::command()],
            vec![
                GetPreviewEnabled::command(),
                TogglePreviewEnabled::command(),
            ],
            vec![Unsubscribe::command()],
        ];

        for command_row in rows {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();

            for command in command_row {
                let button = InlineKeyboardButton::builder()
                    .text(command.to_string())
                    .callback_data(format!("{} {}", command, self.feed_url))
                    .build();

                row.push(button);
            }

            buttons.push(row);
        }

        buttons.push(Close::button_row());

        let keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(buttons)
            .build();

        SendMessageParams::builder()
            .chat_id(self.message.chat.id)
            .text(self.feed_url.clone())
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            .build()
    }
}

impl Command for ShowFeedKeyboard {
    fn response(&self) -> Response {
        match self.fetch_db_connection() {
            Ok(mut connection) => {
                let params = self.feed_keyboard(&mut connection);

                Response::Params(params)
            }
            Err(error_message) => Response::Simple(error_message),
        }
    }

    fn send_message(&self, send_message_params: SendMessageParams) {
        match self.api().send_message_with_params(&send_message_params) {
            Err(error) => {
                error!(
                    "Failed to send a message {:?} {:?}",
                    error, send_message_params
                );
            }

            Ok(_) => {
                self.remove_message(&self.message);
            }
        }
    }
}
