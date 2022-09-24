use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;

pub struct UnsubscribeInlineKeyboard {}
impl UnsubscribeInlineKeyboard {
    pub fn set_unsubscribe_keyboard(
        message: Message,
        feeds: std::str::Split<'_, char>,
        feed_id: String,
    ) -> SendMessageParams {
        let id = feed_id.split("/n");
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for feed in feeds.clone() {
            for feedid in id.clone() {
                let mut row: Vec<InlineKeyboardButton> = Vec::new();
                let name = format!("{} ", feed);

                let unsubscribe_inlinekeyboard = InlineKeyboardButton::builder()
                    .text(name.clone())
                    .callback_data(format!("unsubscribe {}", feedid))
                    .build();

                row.push(unsubscribe_inlinekeyboard);
                keyboard.push(row);
            }
        }

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();

        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select feed url to unsubscribe")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
}