use crate::bot::logic;
use crate::bot::logic::{DeleteSubscriptionError, SubscriptionError};
use crate::bot::telegram_client::Api;
use crate::bot::telegram_client::Error;
use crate::db;
use crate::db::telegram::NewTelegramChat;
use frankenstein::Chat;
use frankenstein::ChatId;
use frankenstein::Message;
use frankenstein::SendMessageParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use once_cell::sync::OnceCell;
use std::env;
use tokio::time;

static SUBSCRIBE: &str = "/subscribe";
static LIST_SUBSCRIPTIONS: &str = "/list_subscriptions";
static SET_TIMEZONE: &str = "/set_timezone";
static GET_TIMEZONE: &str = "/get_timezone";
static SET_TEMPLATE: &str = "/set_template";
static GET_TEMPLATE: &str = "/get_template";
static SET_FILTER: &str = "/set_filter";
static GET_FILTER: &str = "/get_filter";
static REMOVE_FILTER: &str = "/remove_filter";
static SET_GLOBAL_TEMPLATE: &str = "/set_global_template";
static GET_GLOBAL_TEMPLATE: &str = "/get_global_template";
static UNSUBSCRIBE: &str = "/unsubscribe";
static HELP: &str = "/help";
static START: &str = "/start";
static OWNER_TELEGRAM_ID: OnceCell<Option<i64>> = OnceCell::new();

static COMMANDS: [&str; 14] = [
    SUBSCRIBE,
    LIST_SUBSCRIPTIONS,
    SET_TIMEZONE,
    GET_TIMEZONE,
    SET_TEMPLATE,
    GET_TEMPLATE,
    SET_FILTER,
    GET_FILTER,
    REMOVE_FILTER,
    SET_GLOBAL_TEMPLATE,
    GET_GLOBAL_TEMPLATE,
    UNSUBSCRIBE,
    HELP,
    START,
];

impl From<Chat> for NewTelegramChat {
    fn from(chat: Chat) -> Self {
        NewTelegramChat {
            id: chat.id() as i64,
            kind: chat.type_field(),
            username: chat.username(),
            first_name: chat.first_name(),
            last_name: chat.last_name(),
            title: chat.title(),
        }
    }
}

fn commands_string() -> String {
    format!(
        "{} - show the bot's description and contact information\n\n\
         {} url - subscribe to feed\n\n\
         {} url - unsubscribe from feed\n\n\
         {} - list your subscriptions\n\n\
         {} - show available commands\n\n\
         {} - set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, offset is equal to 600\n\n\
         {} - get your timezone\n\n\
         {} url template - set a template for all received items for the specified subscription. All new updates will be converted to the format defined by this subscription. Supported fields you can use for templates:\n\
         - bot_feed_name - name of the feed\n\
         - bot_feed_link - url of the feed\n\
         - bot_item_name - name of the item\n\
         - bot_item_link - url of the item\n\
         - bot_item_description - description of the item\n\
         - bot_date - publication date of the feed\n\
         - bot_space - defines a space character\n\
         - bot_new_line - defines a new line character\n\
         Example: /set_template https://www.badykov.com/feed.xml bot_datebot_spacebot_item_namebot_new_linebot_item_description\n\n\
         {} url - get a template for the subscription\n\n\
         {} template - set global template. This template will be used for all subscriptions. If the subscription has its own template, the subscription template will be used. See /set_template for available fields.\n\n\
         {} - get global template\n\n\
         {} url - get a filter for the subscription\n\n\
         {} url template - set filter, for example, /set_filter https://www.badykov.com/feed.xml telegram,bots. You'll start receiving posts only containing words in the filter. Use `!word` to stop receiving messages containing the specified `word`. You can combine regular filter words with ! filter words. For example, `!bot,telegram`\n\n\
         {} url - remove filter\n\n",
        START, SUBSCRIBE, UNSUBSCRIBE, LIST_SUBSCRIPTIONS, HELP, SET_TIMEZONE, GET_TIMEZONE, SET_TEMPLATE, GET_TEMPLATE, SET_GLOBAL_TEMPLATE, GET_GLOBAL_TEMPLATE, GET_FILTER, SET_FILTER, REMOVE_FILTER
    )
}

async fn help(api: Api, message: Message) -> Result<(), Error> {
    let response_help = commands_string();

    let response = format!(
        "{}\n\
        If you still have questions please join https://t.me/el_monitorro or contact the author (@Ayrat555)\n",
        response_help
    );

    reply_to_message(api, message, response)
}

async fn start(api: Api, message: Message) -> Result<(), Error> {
    let response = format!(
        "El Monitorro is feed reader as a Telegram bot.\n\
         It supports RSS, Atom and JSON feeds.\n\n\
         Available commands:\n\
         {}\n\n\
         Synchronization information.\n\
         When you subscribe to a new feed, you'll receive 10 last messages from it. After that, you'll start receiving only new feed items.\n\
         Feed updates check interval is 1 minute. Unread items delivery interval is also 1 minute.\n\
         Currently, the number of subscriptions is limited to {}.\n\n\
         Join https://t.me/el_monitorro or contact the author (@Ayrat555) with your feedback, suggestions, found bugs, etc. The bot is open source. You can find it at https://github.com/ayrat555/el_monitorro\n\n\
         Unlike other similar projects, El Monitorro is completely open and it's free of charge. I develop it in my free time and pay for hosting myself. Consider donating to the project - https://paypal.me/ayrat555",
        commands_string(),
        logic::sub_limit()
    );

    reply_to_message(api, message, response)
}

pub async fn send_message(chat_id: i64, message: String) -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token);

    let send_message_params = SendMessageParams::new(ChatId::Integer(chat_id), message);

    match api.send_message(&send_message_params) {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!("Failed to send message: {:?}", err);
            Err(err)
        }
    }
}

async fn unknown_command(api: Api, message: Message) -> Result<(), Error> {
    let response = "Unknown command. Use /help to show available commands".to_string();

    reply_to_message(api, message, response)
}

fn reply_to_message(api: Api, message: Message, text: String) -> Result<(), Error> {
    let mut send_message_params =
        SendMessageParams::new(ChatId::Integer(message.chat().id()), text);

    send_message_params.set_reply_to_message_id(Some(message.message_id()));

    match api.send_message(&send_message_params) {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!("Failed to send message: {:?}", err);
            Err(err)
        }
    }
}

async fn subscribe(api: Api, message: Message, data: String) -> Result<(), Error> {
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response =
        match logic::create_subscription(&db_connection, message.chat().into(), Some(data.clone()))
        {
            Ok(_subscription) => format!("Successfully subscribed to {}", data),
            Err(SubscriptionError::DbError(_)) => {
                "Something went wrong with the bot's storage".to_string()
            }
            Err(SubscriptionError::InvalidUrl) => "Invalid url".to_string(),
            Err(SubscriptionError::RssUrlNotProvided) => "Url is not provided".to_string(),
            Err(SubscriptionError::UrlIsNotFeed) => "Url is not a feed".to_string(),
            Err(SubscriptionError::SubscriptionAlreadyExists) => {
                "Susbscription already exists".to_string()
            }
            Err(SubscriptionError::SubscriptionCountLimit) => {
                "You exceeded the number of subscriptions".to_string()
            }
            Err(SubscriptionError::TelegramError) => {
                "Something went wrong with Telegram".to_string()
            }
        };

    reply_to_message(api, message, response)
}

async fn unsubscribe(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = match logic::delete_subscription(&db_connection, chat_id, data.clone()) {
        Ok(_) => format!("Successfully unsubscribed from {}", data),
        Err(DeleteSubscriptionError::DbError) => format!("Failed to unsubscribe from {}", data),
        _ => "Subscription does not exist".to_string(),
    };

    reply_to_message(api, message, response)
}

async fn list_subscriptions(api: Api, message: Message) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::find_feeds_by_chat_id(&db_connection, chat_id);

    reply_to_message(api, message, response)
}

async fn set_timezone(api: Api, message: Message, data: String) -> Result<(), Error> {
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = match logic::set_timezone(&db_connection, message.chat().id() as i64, data) {
        Ok(_) => "Your timezone was updated".to_string(),
        Err(err_string) => err_string.to_string(),
    };

    reply_to_message(api, message, response)
}

async fn set_template(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::set_template(&db_connection, chat_id, data);

    reply_to_message(api, message, response)
}

async fn set_filter(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::set_filter(&db_connection, chat_id, data);

    reply_to_message(api, message, response)
}

async fn get_timezone(api: Api, message: Message) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::get_timezone(&db_connection, chat_id);

    reply_to_message(api, message, response)
}

async fn get_global_template(api: Api, message: Message) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::get_global_template(&db_connection, chat_id);

    reply_to_message(api, message, response)
}

async fn set_global_template(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::set_global_template(&db_connection, chat_id, data);

    reply_to_message(api, message, response)
}

async fn get_template(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::get_template(&db_connection, chat_id, data);

    reply_to_message(api, message, response)
}

async fn get_filter(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::get_filter(&db_connection, chat_id, data);

    reply_to_message(api, message, response)
}

async fn remove_filter(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = message.chat().id() as i64;
    let semaphored_connection = db::get_semaphored_connection().await;
    let db_connection = semaphored_connection.connection;

    let response = logic::remove_filter(&db_connection, chat_id, data);

    reply_to_message(api, message, response)
}

// fn process_message(api: Api, orig_message: Message) {
//     match orig_message.kind {
//         MessageKind::Text { ref data, .. } => {
//             let command = data.clone();
//             let message = MessageOrChannelPost::Message(orig_message.clone());

//             tokio::spawn(process_message_or_channel_post(api, message, command));
//         }
//         _ => (),
//     };
// }

// fn process_channel_post(api: Api, post: ChannelPost) {
//     match post.kind {
//         MessageKind::Text { ref data, .. } => {
//             let command = data.clone();

//             tokio::spawn(process_message_or_channel_post(
//                 api,
//                 MessageOrChannelPost::ChannelPost(post.clone()),
//                 command,
//             ));
//         }
//         _ => (),
//     };
// }

fn owner_telegram_id() -> &'static Option<i64> {
    OWNER_TELEGRAM_ID.get_or_init(|| match env::var("OWNER_TELEGRAM_ID") {
        Ok(val) => {
            let parsed_value: i64 = val.parse().unwrap();
            Some(parsed_value)
        }
        Err(_error) => None,
    })
}

async fn process_message_or_channel_post(api: Api, update: Update) -> Result<(), Error> {
    let message = match update.message() {
        None => update.channel_post().unwrap(),
        Some(message) => message,
    };

    let chat_id = message.chat().id() as i64;

    if let Some(id) = owner_telegram_id() {
        if *id != chat_id {
            return Ok(());
        }
    }

    let text = message.text();

    if text.is_none() {
        return Ok(());
    }

    let command = &text.unwrap();

    let is_known_command = COMMANDS
        .iter()
        .any(|command_name| command.starts_with(command_name));

    if is_known_command {
        log::info!("{:?} wrote: {}", chat_id, command);
    }

    if command.starts_with(SUBSCRIBE) {
        let argument = parse_argument(command, SUBSCRIBE);
        tokio::spawn(subscribe(api, message, argument));
    } else if command.starts_with(LIST_SUBSCRIPTIONS) {
        tokio::spawn(list_subscriptions(api, message));
    } else if command.starts_with(UNSUBSCRIBE) {
        let argument = parse_argument(command, UNSUBSCRIBE);
        tokio::spawn(unsubscribe(api, message, argument));
    } else if command.starts_with(HELP) {
        tokio::spawn(help(api, message));
    } else if command.starts_with(START) {
        tokio::spawn(start(api, message));
    } else if command.starts_with(SET_TIMEZONE) {
        let argument = parse_argument(command, SET_TIMEZONE);
        tokio::spawn(set_timezone(api, message, argument));
    } else if command.starts_with(GET_TIMEZONE) {
        tokio::spawn(get_timezone(api, message));
    } else if command.starts_with(GET_TEMPLATE) {
        let argument = parse_argument(command, GET_TEMPLATE);
        tokio::spawn(get_template(api, message, argument));
    } else if command.starts_with(GET_FILTER) {
        let argument = parse_argument(command, GET_FILTER);
        tokio::spawn(get_filter(api, message, argument));
    } else if command.starts_with(REMOVE_FILTER) {
        let argument = parse_argument(command, REMOVE_FILTER);
        tokio::spawn(remove_filter(api, message, argument));
    } else if command.starts_with(SET_FILTER) {
        let argument = parse_argument(command, SET_FILTER);
        tokio::spawn(set_filter(api, message, argument));
    } else if command.starts_with(SET_TEMPLATE) {
        let argument = parse_argument(command, SET_TEMPLATE);
        tokio::spawn(set_template(api, message, argument));
    } else if command.starts_with(GET_GLOBAL_TEMPLATE) {
        tokio::spawn(get_global_template(api, message));
    } else if command.starts_with(SET_GLOBAL_TEMPLATE) {
        let argument = parse_argument(command, SET_GLOBAL_TEMPLATE);
        tokio::spawn(set_global_template(api, message, argument));
    } else {
        let chat: NewTelegramChat = message.chat().into();

        if chat.kind == "private" {
            tokio::spawn(unknown_command(api, message));
        }
    }

    Ok(())
}

fn parse_argument(full_command: &str, command: &str) -> String {
    let handle = env::var("TELEGRAM_BOT_HANDLE").unwrap_or_else(|_| "".to_string());
    let command_with_handle = format!("{}@{}", command, handle);

    if full_command.starts_with(&command_with_handle) {
        full_command
            .replace(&command_with_handle, "")
            .trim()
            .to_string()
    } else {
        full_command.replace(command, "").trim().to_string()
    }
}

pub async fn start_bot() {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let mut api = Api::new(token);
    // let mut stream = api.stream();

    log::info!("Starting a bot");

    let mut interval = time::interval(std::time::Duration::from_secs(1));

    loop {
        while let Some(update) = api.next_update() {
            tokio::spawn(process_message_or_channel_post(api.clone(), update));
        }

        interval.tick().await;
    }
}
