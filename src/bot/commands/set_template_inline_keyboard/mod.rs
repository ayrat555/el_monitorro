use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use frankenstein::{
    InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyMarkup, SendMessageParams,
};

use crate::bot::handler::Handler;

static SUBSTRING: &str = "substring";
static ITALIC: &str = "italic";
static CREATE_LINK: &str = "create_link";
static BOLD: &str = "bold";

pub struct SetTemplateInlineKeyboard {}
impl SetTemplateInlineKeyboard {
    pub fn set_template_menu_keyboard(message: Message, feed_id: String) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let mut row1: Vec<InlineKeyboardButton> = Vec::new();
        let mut row2: Vec<InlineKeyboardButton> = Vec::new();
        let mut row3: Vec<InlineKeyboardButton> = Vec::new();
        let mut row4: Vec<InlineKeyboardButton> = Vec::new();
        let mut row5: Vec<InlineKeyboardButton> = Vec::new();

        let substring = InlineKeyboardButton::builder()
            .text("Limit number of characters of feed message")
            .callback_data(format!("substring {}", feed_id))
            .build();

        let bold = InlineKeyboardButton::builder()
            .text("Set your feed message 𝐛𝐨𝐥𝐝 ")
            .callback_data(format!("bold {}", feed_id))
            .build();

        let italic = InlineKeyboardButton::builder()
            .text("Set your feed message 𝘪𝘵𝘢𝘭𝘪𝘤")
            .callback_data(format!("italic {}", feed_id))
            .build();

        let create_link = InlineKeyboardButton::builder()
            .text("Create link to feed site")
            .callback_data(format!("create_link {}", feed_id))
            .build();

        let back_to_subscription_list_keyboard = InlineKeyboardButton::builder()
            .text("Back to menu 🔙 ")
            .callback_data(format!("list_subscriptions {}", feed_id)) //used letter s to identify the callback ,callback data support no of characters
            .build();

        row1.push(substring);
        row2.push(bold);
        row3.push(italic);
        row4.push(create_link);
        row5.push(back_to_subscription_list_keyboard);

        keyboard.push(row1);
        keyboard.push(row2);
        keyboard.push(row3);
        keyboard.push(row4);
        keyboard.push(row5);

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();
        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select your option")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
    pub fn select_feed_url_keyboard(
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
                .callback_data(format!("set_template {}", feed)) //used letter s to identify the callback ,callback data support no of characters
                .build();

            row.push(unsubscribe_inlinekeyboard);
            keyboard.push(row);
        }

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();

        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select feed url to modify")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
    pub fn set_template_substring_keyboard(
        message: Message,
        feed_id: String,
        feed_url: String,
    ) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let mut row1: Vec<InlineKeyboardButton> = Vec::new();
        let mut row2: Vec<InlineKeyboardButton> = Vec::new();
        let mut row3: Vec<InlineKeyboardButton> = Vec::new();

        let substring_bot_description = InlineKeyboardButton::builder()
            .text("Limit bot item description characters")
            .switch_inline_query_current_chat(format!(
                "/set_template {} {{substring bot_item_description 100 }}",
                feed_url
            ))
            .build();
        let substring_bot_name = InlineKeyboardButton::builder()
            .text("Limit bot item name characters")
            .switch_inline_query_current_chat(format!(
                "/set_template {} {{substring bot_item_name 100 }}",
                feed_url
            ))
            .build();
        let back_to_menu = InlineKeyboardButton::builder()
            .text("Back to menus 🔙 ")
            .callback_data(format!("set_templates {}", feed_id))
            .build();

        row1.push(substring_bot_description);
        row2.push(substring_bot_name);
        row3.push(back_to_menu);

        keyboard.push(row1);
        keyboard.push(row2);
        keyboard.push(row3);
        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();
        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select your option")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
    pub fn set_template_bold_keyboard(message: Message, feed_id: String) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let mut row1: Vec<InlineKeyboardButton> = Vec::new();
        let mut row2: Vec<InlineKeyboardButton> = Vec::new();
        let mut row3: Vec<InlineKeyboardButton> = Vec::new();

        let bold_bot_description = InlineKeyboardButton::builder()
            .text("Make bot item description 𝐛𝐨𝐥𝐝")
            .callback_data(format!(
                "/set_template {} {{bold bot_item_description}}",
                feed_id
            ))
            .build();
        let bold_bot_item_name = InlineKeyboardButton::builder()
            .text("Make bot item name 𝐛𝐨𝐥𝐝")
            .callback_data(format!("/set_template {} {{bold bot_item_name}}", feed_id))
            .build();
        let back_to_menu = InlineKeyboardButton::builder()
            .text("Back to menu 🔙 ")
            .callback_data(format!("set_template {}", feed_id))
            .build();

        row1.push(bold_bot_description);
        row2.push(bold_bot_item_name);
        row3.push(back_to_menu);

        keyboard.push(row1);
        keyboard.push(row2);
        keyboard.push(row3);

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();

        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select your option")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
    pub fn set_template_italic_keyboard(message: Message, feed_id: String) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let mut row1: Vec<InlineKeyboardButton> = Vec::new();
        let mut row2: Vec<InlineKeyboardButton> = Vec::new();
        let mut row3: Vec<InlineKeyboardButton> = Vec::new();

        let italic_bot_description = InlineKeyboardButton::builder()
            .text("Make bot item description 𝘪𝘵𝘢𝘭𝘪𝘤")
            .callback_data(format!(
                "/set_template {} {{italic bot_item_description}}",
                feed_id
            ))
            .build();
        let italic_bot_item_name = InlineKeyboardButton::builder()
            .text("Make bot item name 𝘪𝘵𝘢𝘭𝘪𝘤")
            .callback_data(format!(
                "/set_template {} {{italic bot_item_name}}",
                feed_id
            ))
            .build();
        let back_to_menu = InlineKeyboardButton::builder()
            .text("Back to menu 🔙 ")
            .callback_data(format!("set_template {}", feed_id))
            .build();

        row1.push(italic_bot_description);
        row2.push(italic_bot_item_name);
        row3.push(back_to_menu);

        keyboard.push(row1);
        keyboard.push(row2);
        keyboard.push(row3);

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();

        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select your option")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }
    pub fn set_template_create_link_keyboard(
        message: Message,
        feed_id: String,
        feed_url: String,
    ) -> SendMessageParams {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let mut row1: Vec<InlineKeyboardButton> = Vec::new();
        let mut row2: Vec<InlineKeyboardButton> = Vec::new();
        let mut row3: Vec<InlineKeyboardButton> = Vec::new();
        let mut row4: Vec<InlineKeyboardButton> = Vec::new();

        let create_link_bot_item_description = InlineKeyboardButton::builder()
            .text("Make bot item description as link")
            .callback_data(format!("/set_template {} create_link_description", feed_id))
            .build();
        let create_link_bot_item_name = InlineKeyboardButton::builder()
            .text("Make bot item name as link")
            .callback_data(format!("/set_template {} create_link_item_name", feed_id))
            .build();
        let create_link_custom_name = InlineKeyboardButton::builder()
            .text("Make custom name as link")
            .switch_inline_query_current_chat(format!(
                "/set_template {} {{create_link \"custom name\" bot_item_link}}",
                feed_url
            ))
            .build();
        let back_to_menu = InlineKeyboardButton::builder()
            .text("Back to menu 🔙 ")
            .callback_data(format!("set_template {}", feed_id))
            .build();

        row1.push(create_link_bot_item_description);
        row2.push(create_link_bot_item_name);
        row3.push(create_link_custom_name);
        row4.push(back_to_menu);

        keyboard.push(row1);
        keyboard.push(row2);
        keyboard.push(row3);
        keyboard.push(row4);

        let inline_keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(keyboard)
            .build();
        SendMessageParams::builder()
            .chat_id(message.chat.id)
            .text("Select your option")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .build()
    }

    pub fn substring() -> &'static str {
        SUBSTRING
    }

    pub fn bold() -> &'static str {
        BOLD
    }

    pub fn create_link() -> &'static str {
        CREATE_LINK
    }

    pub fn italic() -> &'static str {
        ITALIC
    }
}
