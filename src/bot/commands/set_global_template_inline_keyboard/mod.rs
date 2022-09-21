use frankenstein::{
    InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyMarkup, SendMessageParams,
};
pub fn set_global_template_keyboard(message: Message) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();
    let mut row4: Vec<InlineKeyboardButton> = Vec::new();

    let substring = InlineKeyboardButton::builder()
        .text("Limit number of characters of feed message")
        .callback_data("global_substring")
        .build();
    let bold = InlineKeyboardButton::builder()
        .text("Set your feed message bold ")
        .callback_data("global_bold")
        .build();
    let italic = InlineKeyboardButton::builder()
        .text("Set your feed message italic")
        .callback_data("global_italic")
        .build();
    let create_link = InlineKeyboardButton::builder()
        .text("Create link to feed site")
        .callback_data("global_create_link")
        .build();

    row.push(substring);
    row2.push(bold);
    row3.push(italic);
    row4.push(create_link);

    keyboard.push(row);
    keyboard.push(row2);
    keyboard.push(row3);
    keyboard.push(row4);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_substring_keyboard(message: Message) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let substring_bot_description = InlineKeyboardButton::builder()
        .text("Limit bot item description characters")
        .switch_inline_query_current_chat(
            "/set_global_template {{substring bot_item_description 100 }}",
        )
        .build();
    let substring_bot_name = InlineKeyboardButton::builder()
        .text("Limit bot item name characters")
        .switch_inline_query_current_chat("/set_global_template {{substring bot_item_name 100 }}")
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
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
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_bold_keyboard(message: Message) -> SendMessageParams {
    let text = message.text.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let bold_bot_description = InlineKeyboardButton::builder()
        .text("Make bot item description 𝐛𝐨𝐥𝐝")
        .callback_data("/set_global_template {{bold bot_item_description }}")
        .build();
    let bold_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name 𝐛𝐨𝐥𝐝")
        .callback_data("/set_global_template {{bold bot_item_name }}")
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
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
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_italic_keyboard(message: Message) -> SendMessageParams {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let text = message.text.unwrap();
    println!(
        "text inside set global templ italic keyboard ======{:?}",
        text
    );
    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();

    let italic_bot_item_description = InlineKeyboardButton::builder()
        .text("Make bot item description 𝘪𝘵𝘢𝘭𝘪𝘤")
        .callback_data("/set_global_template {{italic bot_item_description }}")
        .build();
    let italic_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name 𝘪𝘵𝘢𝘭𝘪𝘤")
        .callback_data("/set_global_template {{italic bot_item_name }}")
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
        .build();

    row.push(italic_bot_item_description);
    row2.push(italic_bot_item_name);
    row3.push(back_to_menu);

    keyboard.push(row);
    keyboard.push(row2);
    keyboard.push(row3);

    let inline_keyboard = InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build();
    SendMessageParams::builder()
        .chat_id(message.chat.id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}

pub fn set_global_template_create_link_keyboard(message: Message) -> SendMessageParams {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let text = message.text.unwrap();

    let mut row1: Vec<InlineKeyboardButton> = Vec::new();
    let mut row2: Vec<InlineKeyboardButton> = Vec::new();
    let mut row3: Vec<InlineKeyboardButton> = Vec::new();
    let mut row4: Vec<InlineKeyboardButton> = Vec::new();

    let create_link_bot_item_description = InlineKeyboardButton::builder()
        .text("Make bot item description as link")
        .callback_data("/set_global_template create_link_description")
        .build();
    let create_link_bot_item_name = InlineKeyboardButton::builder()
        .text("Make bot item name as link")
        .callback_data("/set_global_template create_link_item_name")
        .build();
    let create_link_custom_name = InlineKeyboardButton::builder()
        .text("Make custom name as link")
        .switch_inline_query_current_chat(
            "/set_global_template {{create_link \"custom name\" bot_item_link}}",
        )
        .build();
    let back_to_menu = InlineKeyboardButton::builder()
        .text("Back to menu 🔙 ")
        .callback_data("back to menu")
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
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}
