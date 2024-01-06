use super::Close;
use super::Command;
use super::GetGlobalFilter;
use super::GetGlobalTemplate;
use super::GetPreviewEnabled;
use super::GetTimezone;
use super::Help;
use super::ListSubscriptionsKeyboard;
use super::RemoveGlobalFilter;
use super::RemoveGlobalTemplate;
use super::Response;
use super::SetGlobalFilter;
use super::SetGlobalTemplate;
use super::SetTimezone;
use super::Start;
use super::Subscribe;
use super::TogglePreviewEnabled;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/commands";

#[derive(TypedBuilder)]
pub struct CommandsKeyboard {
    message: Message,
}

impl CommandsKeyboard {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    fn keyboard(&self) -> SendMessageParams {
        let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let rows = [
            vec![
                ("List subscriptions", ListSubscriptionsKeyboard::command()),
                ("Subscribe", Subscribe::command()),
            ],
            vec![
                ("Get global filter", GetGlobalFilter::command()),
                ("Set global filter", SetGlobalFilter::command()),
                ("Remove global filter", RemoveGlobalFilter::command()),
            ],
            vec![
                ("Get global template", GetGlobalTemplate::command()),
                ("Set global template", SetGlobalTemplate::command()),
                ("Remove global template", RemoveGlobalTemplate::command()),
            ],
            vec![
                ("Get timezone", GetTimezone::command()),
                ("Set timezone", SetTimezone::command()),
            ],
            vec![
                (
                    "Check if previews are enabled",
                    GetPreviewEnabled::command(),
                ),
                ("Toggle previews", TogglePreviewEnabled::command()),
            ],
            vec![("Help", Help::command()), ("Start", Start::command())],
        ];

        for command_row in rows {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();

            for (text, command) in command_row {
                let button = InlineKeyboardButton::builder()
                    .text(text)
                    .callback_data(command)
                    .build();

                row.push(button);
            }

            buttons.push(row);
        }

        buttons.push(Close::button_row());

        let keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(buttons)
            .build();

        let mut params = SendMessageParams::builder()
            .chat_id(self.message.chat.id)
            .text("Select a command")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            .build();

        params.message_thread_id = self.message.message_thread_id;

        params
    }
}

impl Command for CommandsKeyboard {
    fn response(&self) -> Response {
        let params = self.keyboard();

        Response::Params(Box::new(params))
    }
}
