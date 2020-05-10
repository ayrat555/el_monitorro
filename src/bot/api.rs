use crate::bot::logic;
use crate::bot::logic::{DeleteSubscriptionError, SubscriptionError};
use crate::db;
use crate::db::telegram::NewTelegramChat;
use futures::StreamExt;
use std::env;
use telegram_bot::prelude::*;
use telegram_bot::{Api, Error, Message, MessageChat, MessageKind, UpdateKind, UserId};

static SUBSCRIBE: &str = "/subscribe";
static LIST_SUBSCRIPTIONS: &str = "/list_subscriptions";
static UNSUBSCRIBE: &str = "/unsubscribe";
static HELP: &str = "/help";

impl From<MessageChat> for NewTelegramChat {
    fn from(message_chat: MessageChat) -> Self {
        if let MessageChat::Private(chat) = message_chat {
            NewTelegramChat {
                id: chat.id.into(),
                kind: "private".to_string(),
                username: chat.username,
                first_name: Some(chat.first_name),
                last_name: chat.last_name,
            }
        } else {
            unimplemented!()
        }
    }
}

async fn help(api: Api, message: Message) -> Result<(), Error> {
    let response = format!("{} rss_url - subscribe to rss feed\n{} rss_url - unsubscribe from rss feed\n{} - list your subscriptions\n{} - show available commands", SUBSCRIBE, UNSUBSCRIBE, LIST_SUBSCRIPTIONS, HELP);

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

async fn unknown_command(api: Api, message: Message) -> Result<(), Error> {
    let response = "Unknown command. Use /help to show available commands".to_string();

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn subscribe(api: Api, message: Message, data: String) -> Result<(), Error> {
    let response = match logic::create_subscription(
        &db::establish_connection(),
        message.chat.clone().into(),
        Some(data.clone()),
    ) {
        Ok(_subscription) => format!("Successfully subscribed to {}", data),
        Err(SubscriptionError::DbError(_)) => {
            "Something went wrong with the bot's storage".to_string()
        }
        Err(SubscriptionError::InvalidRssUrl) => "Invalid url".to_string(),
        Err(SubscriptionError::RssUrlNotProvided) => "Url is not provided".to_string(),
        Err(SubscriptionError::UrlIsNotRss) => "Url is not rss feed".to_string(),
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

async fn unsubscribe(api: Api, message: Message, data: String) -> Result<(), Error> {
    let chat_id = if let MessageChat::Private(chat) = &message.chat {
        chat.id
    } else {
        unimplemented!()
    };
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

async fn list_subscriptions(api: Api, message: Message) -> Result<(), Error> {
    let chat_id = if let MessageChat::Private(chat) = &message.chat {
        chat.id
    } else {
        unimplemented!()
    };

    let response = logic::find_feeds_by_chat_id(&db::establish_connection(), chat_id.into());

    api.send(message.text_reply(response)).await?;
    Ok(())
}

async fn process(api: Api, message: Message) -> Result<(), Error> {
    match message.kind {
        MessageKind::Text { ref data, .. } => {
            let command = data.as_str();

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
            } else {
                tokio::spawn(unknown_command(api, message));
            }
        }
        _ => (),
    };

    Ok(())
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
        if let UpdateKind::Message(message) = update.kind {
            process(api.clone(), message).await?;
        }
    }

    Ok(())
}
