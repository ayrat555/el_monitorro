use crate::bot::logic;
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
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let known = bot
        .new_cmd("/subscribe")
        .and_then(|(bot, msg)| {
            let chat = NewTelegramChat::from(msg.chat);
            let result =
                logic::create_subscription(&db::establish_connection(), chat, msg.text).unwrap();
            bot.message(result.chat_id, format!("{:?}", result).into())
                .send()
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

impl From<Chat> for NewTelegramChat {
    fn from(chat: Chat) -> Self {
        NewTelegramChat {
            id: chat.id,
            kind: chat.kind,
            title: chat.title,
            username: chat.username,
            first_name: chat.first_name,
            last_name: chat.last_name,
        }
    }
}
