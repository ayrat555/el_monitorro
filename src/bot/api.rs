use crate::bot::logic;
use crate::bot::logic::{DeleteSubscriptionError, SubscriptionError};
use crate::db;
use crate::db::telegram::NewTelegramChat;
use futures::StreamExt;
use std::env;
use telegram_bot::prelude::*;
use telegram_bot::{
    Api, ChannelPost, Error, Message, MessageChat, MessageKind, MessageOrChannelPost, UpdateKind,
    UserId,
};

static SUBSCRIBE: &str = "/subscribe";
static LIST_SUBSCRIPTIONS: &str = "/list_subscriptions";
static SET_TIMEZONE: &str = "/set_timezone";
static GET_TIMEZONE: &str = "/get_timezone";
static SET_TEMPLATE: &str = "/set_template";
static GET_TEMPLATE: &str = "/get_template";
static UNSUBSCRIBE: &str = "/unsubscribe";
static HELP: &str = "/help";
static START: &str = "/start";

impl From<MessageChat> for NewTelegramChat {
    fn from(message_chat: MessageChat) -> Self {
        match message_chat {
            MessageChat::Private(chat) => NewTelegramChat {
                id: chat.id.into(),
                kind: "private".to_string(),
                username: chat.username,
                first_name: Some(chat.first_name),
                last_name: chat.last_name,
                title: None,
            },
            MessageChat::Group(chat) => NewTelegramChat {
                id: chat.id.into(),
                kind: "group".to_string(),
                title: Some(chat.title),
                username: None,
                first_name: None,
                last_name: None,
            },
            MessageChat::Supergroup(chat) => NewTelegramChat {
                id: chat.id.into(),
                kind: "supergroup".to_string(),
                title: Some(chat.title),
                username: chat.username,
                first_name: None,
                last_name: None,
            },
            MessageChat::Unknown(chat) => NewTelegramChat {
                id: chat.id.into(),
                kind: "unknown".to_string(),
                title: chat.title,
                username: chat.username,
                first_name: chat.first_name,
                last_name: chat.last_name,
            },
        }
    }
}

impl From<MessageOrChannelPost> for NewTelegramChat {
    fn from(message_or_post: MessageOrChannelPost) -> Self {
        match message_or_post {
            MessageOrChannelPost::Message(message) => message.chat.into(),
            MessageOrChannelPost::ChannelPost(post) => NewTelegramChat {
                id: post.chat.id.into(),
                kind: "channel".to_string(),
                title: Some(post.chat.title),
                username: post.chat.username,
                first_name: None,
                last_name: None,
            },
        }
    }
}

fn commands_string() -> String {
    format!(
        "{} - show the bot's description and contact information\n\
         {} url - subscribe to feed\n\
         {} url - unsubscribe from feed\n\
         {} - list your subscriptions\n\
         {} - show available commands\n\
         {} - set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, offset is equal to 600\n\
         {} - get your timezone\n",
        START, SUBSCRIBE, UNSUBSCRIBE, LIST_SUBSCRIPTIONS, HELP, SET_TIMEZONE, GET_TIMEZONE
    )
}

async fn help(api: Api, message: MessageOrChannelPost) -> Result<(), Error> {
    let response = commands_string();

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn start(api: Api, message: MessageOrChannelPost) -> Result<(), Error> {
    let response = format!(
        "El Monitorro is feed reader as a Telegram bot.\n\
         It supports RSS, Atom and JSON feeds.\n\n\
         Available commands:\n\
         {}\n\n\
         Synchronization information.\n\
         When you subscribe to a new feed, you'll receive 10 last messages from it. After that, you'll start receiving only new feed items.\n\
         Feed updates check interval is 1 minute. Unread items delivery interval is also 1 minute.\n\
         Currently, the number of subscriptions is limited to 20.\n\n\
         Contact @Ayrat555 with your feedback, suggestions, found bugs, etc. The bot is open source. You can find it at https://github.com/ayrat555/el_monitorro\n\n\
         Unlike other similar projects, El Monitorro is completely open and it's free of charge. I develop it in my free time and pay for hosting myself. Consider donating to the project - https://paypal.me/ayrat555",
        commands_string()
    );

    api.send(message.text_reply(response)).await?;
    Ok(())
}

pub async fn send_message(chat_id: i64, message: String) -> Result<(), Error> {
    let user_id: UserId = chat_id.into();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token);

    api.send(user_id.text(message)).await?;

    Ok(())
}

async fn unknown_command(api: Api, message: MessageOrChannelPost) -> Result<(), Error> {
    let response = "Unknown command. Use /help to show available commands".to_string();

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn subscribe(api: Api, message: MessageOrChannelPost, data: String) -> Result<(), Error> {
    let response = match logic::create_subscription(
        &db::establish_connection(),
        message.clone().into(),
        Some(data.clone()),
    ) {
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
        Err(SubscriptionError::TelegramError) => "Something went wrong with Telegram".to_string(),
    };

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn unsubscribe(api: Api, message: MessageOrChannelPost, data: String) -> Result<(), Error> {
    let chat_id = get_chat_id(&message);

    let response =
        match logic::delete_subscription(&db::establish_connection(), chat_id.into(), data.clone())
        {
            Ok(_) => format!("Successfully unsubscribed from {}", data),
            Err(DeleteSubscriptionError::DbError) => format!("Failed to unsubscribe from {}", data),
            _ => "Subscription does not exist".to_string(),
        };

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn list_subscriptions(api: Api, message: MessageOrChannelPost) -> Result<(), Error> {
    let chat_id = get_chat_id(&message);

    let response = logic::find_feeds_by_chat_id(&db::establish_connection(), chat_id.into());

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn set_timezone(api: Api, message: MessageOrChannelPost, data: String) -> Result<(), Error> {
    let chat_id = get_chat_id(&message);

    let response = match logic::set_timezone(&db::establish_connection(), chat_id, data) {
        Ok(_) => "Your timezone was updated".to_string(),
        Err(err_string) => err_string.to_string(),
    };

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn set_template(api: Api, message: MessageOrChannelPost, data: String) -> Result<(), Error> {
    let chat_id = get_chat_id(&message);

    let response = match logic::set_template(&db::establish_connection(), chat_id, data) {
        Ok(_) => "The template was updated".to_string(),
        Err(err_string) => err_string.to_string(),
    };

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn get_timezone(api: Api, message: MessageOrChannelPost) -> Result<(), Error> {
    let chat_id = get_chat_id(&message);

    let response = logic::get_timezone(&db::establish_connection(), chat_id);

    api.send(message.text_reply(response)).await?;
    Ok(())
}

fn process_message(api: Api, orig_message: Message) {
    match orig_message.kind {
        MessageKind::Text { ref data, .. } => {
            let command = data.clone();
            let message = MessageOrChannelPost::Message(orig_message.clone());

            log::info!("{:?} wrote: {}", get_chat_id(&message), command);

            tokio::spawn(process_message_or_channel_post(api, message, command));
        }
        _ => (),
    };
}

fn process_channel_post(api: Api, post: ChannelPost) {
    match post.kind {
        MessageKind::Text { ref data, .. } => {
            let command = data.clone();

            tokio::spawn(process_message_or_channel_post(
                api,
                MessageOrChannelPost::ChannelPost(post.clone()),
                command,
            ));
        }
        _ => (),
    };
}

async fn process_message_or_channel_post(
    api: Api,
    message: MessageOrChannelPost,
    command_string: String,
) -> Result<(), Error> {
    let command = &command_string;

    if command.contains(SUBSCRIBE) {
        let argument = parse_argument(command, SUBSCRIBE);
        tokio::spawn(subscribe(api, message, argument));
    } else if command.contains(LIST_SUBSCRIPTIONS) {
        tokio::spawn(list_subscriptions(api, message));
    } else if command.contains(UNSUBSCRIBE) {
        let argument = parse_argument(command, UNSUBSCRIBE);
        tokio::spawn(unsubscribe(api, message, argument));
    } else if command.contains(HELP) {
        tokio::spawn(help(api, message));
    } else if command.contains(START) {
        tokio::spawn(start(api, message));
    } else if command.contains(SET_TIMEZONE) {
        let argument = parse_argument(command, SET_TIMEZONE);
        tokio::spawn(set_timezone(api, message, argument));
    } else if command.contains(GET_TIMEZONE) {
        tokio::spawn(get_timezone(api, message));
    } else {
        if let MessageOrChannelPost::Message(_) = message {
            tokio::spawn(unknown_command(api, message));
        }
    }

    Ok(())
}

fn get_chat_id(message: &MessageOrChannelPost) -> i64 {
    match message {
        MessageOrChannelPost::Message(message) => message.chat.id().into(),
        MessageOrChannelPost::ChannelPost(post) => post.chat.id.into(),
    }
}

fn parse_argument(full_command: &str, command: &str) -> String {
    full_command.replace(command, "").trim().to_string()
}

pub async fn start_bot() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token);
    let mut stream = api.stream();

    log::info!("Starting a bot");

    while let Some(update) = stream.next().await {
        let update = update?;
        match update.kind {
            UpdateKind::Message(message) => {
                process_message(api.clone(), message);
            }
            UpdateKind::ChannelPost(message) => {
                process_channel_post(api.clone(), message);
            }
            _ => (),
        }
    }

    Ok(())
}
