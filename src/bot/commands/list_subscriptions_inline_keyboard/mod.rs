use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use frankenstein::{
    InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyMarkup, SendMessageParams,
};

use crate::bot::handler::Handler;
static BACK_TO_MENU: &str = "back to menu";

pub struct ListSubscriptionsInlineKeyboard {}
impl ListSubscriptionsInlineKeyboard {
    pub fn set_list_subcriptions_menu_keyboard(
        message: Message,
        feed_id: String,
        _feed_url: String,
    ) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let mut row1: Vec<InlineKeyboardButton> = Vec::new();
        let mut row2: Vec<InlineKeyboardButton> = Vec::new();
        let mut row3: Vec<InlineKeyboardButton> = Vec::new();
        let mut row4: Vec<InlineKeyboardButton> = Vec::new();
        let mut row5: Vec<InlineKeyboardButton> = Vec::new();
        let mut row7: Vec<InlineKeyboardButton> = Vec::new();

        let unsubscribe = InlineKeyboardButton::builder()
            .text("Unsubscribe")
            .callback_data(format!("/unsubscribe {}", feed_id))
            .build();

        let remove_filter = InlineKeyboardButton::builder()
            .text("Remove filter ")
            .callback_data(format!("/remove_filter {}", feed_id))
            .build();

        let get_filter = InlineKeyboardButton::builder()
            .text("Get filter ")
            .callback_data(format!("/get_filter {}", feed_id))
            .build();

        let set_template = InlineKeyboardButton::builder()
            .text("Set template")
            .callback_data(format!("set_template {}", feed_id))
            .build();

        let remove_template = InlineKeyboardButton::builder()
            .text("Remove template")
            .callback_data(format!("/remove_template {}", feed_id))
            .build();

        let get_template = InlineKeyboardButton::builder()
            .text("Get template")
            .callback_data(format!("/get_template {}", feed_id))
            .build();

        let set_default_template = InlineKeyboardButton::builder()
            .text("Set default template")
            .callback_data(format!("set_default_template {}", feed_id))
            .build();

        let back_to_menu = InlineKeyboardButton::builder()
            .text("Back to menu 🔙 ")
            .callback_data("/list_subscriptions")
            .build();

        row1.push(set_template);
        row2.push(set_default_template);
        row3.push(get_filter);
        row3.push(remove_filter);
        row4.push(get_template);
        row4.push(remove_template);
        row5.push(unsubscribe);
        row7.push(back_to_menu);

        keyboard.push(row1);
        keyboard.push(row2);
        keyboard.push(row3);
        keyboard.push(row4);
        keyboard.push(row5);
        keyboard.push(row7);

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();
        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select your option")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
    pub fn select_feed_url_keyboard_list_subscriptions(
        message: Message,
        feed_ids: std::str::Split<'_, char>,
        db_pool: Pool<ConnectionManager<PgConnection>>,
    ) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for feed in feed_ids.clone() {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();
            let name = format!(
                "{} ",
                Handler::get_feed_url_by_id(db_pool.clone(), feed.to_string())
            );
            let unsubscribe_inlinekeyboard = InlineKeyboardButton::builder()
                .text(name.clone())
                .callback_data(format!("list_subscriptions {}", feed)) //used letter s to identify the callback ,callback data support no of characters
                .build();

            row.push(unsubscribe_inlinekeyboard);
            keyboard.push(row);
        }

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();

        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select a feed url ")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }

    pub fn back_to_menu() -> &'static str {
        BACK_TO_MENU
    }
}