use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;

pub struct SubscribeInlineKeyboard {}
impl SubscribeInlineKeyboard {
    pub fn set_subscribe_keyboard(message: Message) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        let mut row: Vec<InlineKeyboardButton> = Vec::new();

        let subscribe_inline_keyboard = InlineKeyboardButton::builder()
            .text("Subscribe to a feed")
            .switch_inline_query_current_chat("/subscribe \"put feed link here\"")
            .build();

        row.push(subscribe_inline_keyboard);

        keyboard.push(row);

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();

        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Type feed url to subscribe")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
}
