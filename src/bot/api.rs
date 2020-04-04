use crate::bot::logic;
use crate::bot::logic::SubscriptionError;
use crate::db;
use crate::db::telegram::NewTelegramChat;
use dotenv::dotenv;
use futures::{stream::Stream, Future};
use std::env;
use telebot::objects::Chat;
use telebot::Bot;

use telebot::functions::*;

pub fn start_bot() {
    log::info!("Starting bot");
    dotenv().ok();

    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let known = bot
        .new_cmd("/subscribe")
        .then(|result| {
            let (bot, msg) = result.expect("Telegram error");

            let chat = NewTelegramChat::from(&msg.chat);

            match logic::create_subscription(&db::establish_connection(), chat, msg.text.clone()) {
                Ok(subscription) => Ok((bot, msg, subscription)),
                Err(error) => Err((bot, msg, error)),
            }
        })
        .and_then(|(bot, msg, subscription)| {
            bot.message(msg.chat.id, format!("{:?}", subscription).into())
                .send()
                .map_err(|_err| (bot, msg, SubscriptionError::TelegramError))
        })
        .or_else(|(bot, msg, err)| {
            let text = {
                match err {
                    SubscriptionError::DbError(_) => "Something went wrong with the bot's storage",
                    SubscriptionError::InvalidRssUrl => "Invalid url",
                    SubscriptionError::RssUrlNotProvided => "Url is not provided",
                    SubscriptionError::SubscriptionAlreadyExists => "Susbscription already exists",
                    SubscriptionError::SubscriptionCountLimit => {
                        "You exceeded the number of subscriptins"
                    }
                    SubscriptionError::TelegramError => "Something went wrong with Telegram",
                }
            };

            bot.message(msg.chat.id, text.into()).send()
        })
        .for_each(|_| Ok(()));

    // Every possible command is unknown
    let unknown = bot
        .unknown_cmd()
        .and_then(|(bot, msg)| bot.message(msg.chat.id, "Unknown command".into()).send())
        .for_each(|_| Ok(()));

    // Enter the main loop
    bot.run_with(known.join(unknown));
}

impl From<&Chat> for NewTelegramChat {
    fn from(chat: &Chat) -> Self {
        NewTelegramChat {
            id: chat.id.clone(),
            kind: chat.kind.clone(),
            title: chat.title.clone(),
            username: chat.username.clone(),
            first_name: chat.first_name.clone(),
            last_name: chat.last_name.clone(),
        }
    }
}
