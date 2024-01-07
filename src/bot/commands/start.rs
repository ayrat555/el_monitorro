use super::Command;
use super::CommandsKeyboard;
use super::Message;
use super::Response;
use frankenstein::ChatType;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::LinkPreviewOptions;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static START: &str =
        "El Monitorro is feed reader as a Telegram bot.\n\
         It supports RSS, Atom and JSON feeds.\n\n\
         Use /help to see available commands.\n\n\
         Synchronization information.\n\
         When you subscribe to a new feed, you'll receive 10 last messages from it. After that, you'll start receiving only new feed items.\n\
         Feed updates check interval is 1 minute. Unread items delivery interval is also 1 minute.\n\
         Currently, the number of subscriptions is limited to 20.\n\n\
         Join https://t.me/el_monitorro with your feedback, suggestions, found bugs, etc. The bot is open source. You can find it at https://github.com/ayrat555/el_monitorro\n\n\
         Unlike other similar projects, El Monitorro is completely open and it's free of charge. I develop it in my free time and pay for hosting myself. Please support the bot by donating - https://paypal.me/AyratBadykov, BTC - bc1q94ru65c8pg87ghhjlc7fteuxncpyj8e28cxf42";

static COMMAND: &str = "/start";

#[derive(TypedBuilder)]
pub struct Start {
    message: Message,
}

impl Start {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for Start {
    fn response(&self) -> Response {
        let response = START.to_string();

        if let ChatType::Private = self.message.chat.type_field {
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
            let mut row: Vec<InlineKeyboardButton> = Vec::new();

            let button = InlineKeyboardButton::builder()
                .text("Commands")
                .callback_data(CommandsKeyboard::command())
                .build();

            row.push(button);
            buttons.push(row);

            let keyboard = InlineKeyboardMarkup::builder()
                .inline_keyboard(buttons)
                .build();

            let preview_params = LinkPreviewOptions::builder().is_disabled(true).build();

            let params = SendMessageParams::builder()
                .chat_id(self.message.chat.id)
                .link_preview_options(preview_params)
                .text(response)
                .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
                .build();

            Response::Params(Box::new(params))
        } else {
            Response::Simple(START.to_string())
        }
    }
}
