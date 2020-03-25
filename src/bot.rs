use dotenv::dotenv;
use futures::{stream::Stream, Future};
use std::env;
use telebot::Bot;

use telebot::functions::*;

pub fn start_bot() {
    log::info!("Starting bot");
    dotenv().ok();
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_KEY").unwrap()).update_interval(200);

    let known = bot
        .new_cmd("/known")
        .and_then(|(bot, msg)| {
            println!("{:?}", msg);
            bot.message(msg.chat.id, "This one is known".into()).send()
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
