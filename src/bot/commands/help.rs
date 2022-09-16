use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::KeyboardButton;
use frankenstein::ReplyKeyboardMarkup;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;

static HELP: &str =
        "/start - show the description of the bot and its contact information\n\n\
         /subscribe url - subscribe to the feed\n\n\
         /unsubscribe url - unsubscribe from the feed\n\n\
         /list_subscriptions - list your subscriptions\n\n\
         /help - show available commands\n\n\
         /set_timezone - set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, your offset is equal to 60 x 10 = 600\n\n\
         /get_timezone - get your timezone\n\n\
         /set_template url template - set a template for all received feed items for the specified subscription. All new updates will be converted to the format defined by this subscription. Supported fields you can use for templates:\n\
         - bot_feed_name - name of the feed\n\
         - bot_feed_link - url of the feed\n\
         - bot_item_name - name of the item\n\
         - bot_item_link - url of the item\n\
         - bot_item_description - description of the item\n\
         - bot_date - publication date of the feed\n\
         Example: /set_template https://www.badykov.com/feed.xml {{bot_feed_name}}\n\n\n{{bot_item_name}}\n\n\n{{bot_date}}\n\n\n{{bot_item_link}}\n\n\
         Also, there are some helpers for templates:\n\n\
         - `substring` helper that can be used to limit the number of characters. For example, {{substring bot_item_description 100}}\n\
         - `create_link` helper. This helper creates an html link. For example, {{create_link bot_item_name bot_item_link}} or {{create_link \"custom_name\" bot_item_link}}\n\
         - `italic` helper. Usage: {{italic bot_item_description}}\n\
         - `bold` helper. Usage:  {{bold bot_item_name}}\n\n\
         /get_template url - get the template for the subscription\n\n\
         /remove_template url - remove the template\n\n\
         /set_global_template template - set global template. This template will be used for all subscriptions. If the subscription has its own template, it will be used instead. See /set_template for available fields.\n\n\
         /remove_global_template - remove global template\n\n\
         /get_global_template - get global template\n\n\
         /get_filter url - get the filter for the subscription\n\n\
         /set_filter url template - set filter, for example, /set_filter https://www.badykov.com/feed.xml telegram,bots. You'll start receiving posts only containing words in the filter. Use `!word` to stop receiving messages containing the specified `word`. You can combine regular filter words with ! filter words. For example, `!bot,telegram`\n\n\
         /remove_filter url - remove filter\n\n\
         /set_global_filter filter - set global filter\n\n\
         /get_global_filter - get a global filter\n\n\
         /remove_global_filter - remove global filter\n\n";

static COMMAND: &str = "/help";

pub struct Help {}

impl Help {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for Help {
    fn response(
        &self,
        _db_pool: Pool<ConnectionManager<PgConnection>>,
        _message: &Message,
        _api: &Api,
    ) -> String {
        HELP.to_string()
    }

    fn command(&self) -> &str {
        Self::command()
    }
}

pub fn set_help_keyboard(chatid: i64) -> SendMessageParams {
    // let chat_id: i64 = chatid;
    let mut keyboard: Vec<Vec<KeyboardButton>> = Vec::new();

    let mut row: Vec<KeyboardButton> = Vec::new();
    let mut row2: Vec<KeyboardButton> = Vec::new();
    let mut row3: Vec<KeyboardButton> = Vec::new();
    let mut row4: Vec<KeyboardButton> = Vec::new();
    let mut row5: Vec<KeyboardButton> = Vec::new();
    let mut row6: Vec<KeyboardButton> = Vec::new();

    let start = KeyboardButton::builder()
        .text("/start")
        // .url(
        //     "https://api.telegram.org/bot5523781029:AAEvkb0Cb904Yijt_v698ddhf77OnnBt78I/sendMessage?chat_id=614443505&text=_&reply_markup={%22inline_keyboard%22:%20[[{%22text%22:%20%22/set_global_template%22,%20%22callback_data%22:%20%22hi%22}]]}",
        // )
        .build();
    let subscribe = KeyboardButton::builder()
        // .text("Subscribe to a feed")
        // .switch_inline_query_current_chat
        .text("/subscribe")
        .build();
    let unsubscribe = KeyboardButton::builder()
        // .text("Unsubscribe from a feed")
        // .switch_inline_query_current_chat
        .text("/unsubscribe")
        .build();

    let list_subscription = KeyboardButton::builder()
        // .text("List your subscriptions")
        // .switch_inline_query_current_chat
        .text("/list_subscriptions")
        .build();
    // CallbackQuery::builder().inline_message_id(list_subscription).build();
    let set_global_template = KeyboardButton::builder()
        // .text("Set global template")
        // .switch_inline_query_current_chat
        .text("/set_global_template")
        .build();
    let remove_global_template = KeyboardButton::builder()
        // .text("Remove global template")
        // .switch_inline_query_current_chat
        .text("/remove_global_template")
        .build();
    let set_timezone = KeyboardButton::builder()
        // .text("Set your timezone")
        // .switch_inline_query_current_chat
        .text("/set_timezone")
        .build();
    let get_timezone = KeyboardButton::builder()
        // .text("Get your timezone")
        // .switch_inline_query_current_chat
        .text("/get_timezone")
        .build();
    let set_template = KeyboardButton::builder()
        // .text("Set template")
        // .switch_inline_query_current_chat
        .text("/set_template")
        .build();
    let get_template = KeyboardButton::builder()
        // .text("Get template")
        // .switch_inline_query_current_chat
        .text("/get_template")
        .build();

    row.push(start);
    row2.push(subscribe);
    row2.push(unsubscribe);
    row3.push(list_subscription);
    row4.push(set_global_template);
    row4.push(remove_global_template);
    row5.push(set_timezone);
    row5.push(get_timezone);
    row6.push(set_template);
    row6.push(get_template);

    keyboard.push(row);
    keyboard.push(row2);
    keyboard.push(row3);
    keyboard.push(row4);
    keyboard.push(row5);
    keyboard.push(row6);

    let inline_keyboard = ReplyKeyboardMarkup::builder()
        .keyboard(keyboard)
        .one_time_keyboard(true)
        .resize_keyboard(true)
        .input_field_placeholder("use this to play with your bot")
        .build();

    SendMessageParams::builder()
        .chat_id(chatid)
        .text("Shows all your commands")
        .reply_markup(ReplyMarkup::ReplyKeyboardMarkup(inline_keyboard))
        .build()
}
pub fn send_message_params_builder(
    inline_keyboard: InlineKeyboardMarkup,
    chatid: i64,
    command: String,
) -> SendMessageParams {
    //  let chat_id: i64 = chatid;
    SendMessageParams::builder()
        .chat_id(chatid)
        .text(command)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
        .build()
}
pub fn set_subscribe_keyboard() -> InlineKeyboardMarkup {
    // let chat_id: i64 = 614443505;
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    let italic_bot_item_description = InlineKeyboardButton::builder()
        .text("Subscribe to a feed")
        .switch_inline_query_current_chat("/subscribe \"put feed link here\"")
        .build();

    row.push(italic_bot_item_description);

    keyboard.push(row);

    InlineKeyboardMarkup::builder()
        .inline_keyboard(keyboard)
        .build()
}
