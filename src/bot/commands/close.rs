use super::Command;
use super::Message;
use super::Response;
use frankenstein::InlineKeyboardButton;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/close";
static BUTTON_NAME: &str = "✖ Close";

#[derive(TypedBuilder)]
pub struct Close {
    message: Message,
}

impl Close {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    pub fn button_row() -> Vec<InlineKeyboardButton> {
        let button = InlineKeyboardButton::builder()
            .text(BUTTON_NAME)
            .callback_data(COMMAND)
            .build();

        vec![button]
    }
}

impl Command for Close {
    fn execute(&self, message: &Message, command: &str) {
        info!(
            "{:?} wrote: closed a keyboard - {}",
            message.chat.id, command
        );

        self.remove_message(&self.message);
    }

    // placeholder, not used
    fn response(&self) -> Response {
        Response::Simple("".to_string())
    }
}
