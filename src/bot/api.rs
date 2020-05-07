// use crate::bot::logic;
// use crate::bot::logic::SubscriptionError;
// use crate::db;
// use crate::db::telegram::NewTelegramChat;
use std::env;

use futures::StreamExt;
use telegram_bot::*;

#[tokio::main]
pub async fn start_bot() -> Result<(), Error> {
    // match err {
    //     SubscriptionError::DbError(_) => "Something went wrong with the bot's storage",
    //     SubscriptionError::InvalidRssUrl => "Invalid url",
    //     SubscriptionError::RssUrlNotProvided => "Url is not provided",
    //     SubscriptionError::SubscriptionAlreadyExists => "Susbscription already exists",
    //     SubscriptionError::SubscriptionCountLimit => {
    //         "You exceeded the number of subscriptins"
    //     }
    //     SubscriptionError::TelegramError => "Something went wrong with Telegram",
    // }

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Answer message with "Hi".
                api.send(message.text_reply(format!(
                    "Hi, {}! You just wrote '{}'",
                    &message.from.first_name, data
                )))
                .await?;
            }
        }
    }

    Ok(())
}

// impl From<&Chat> for NewTelegramChat {
//     fn from(chat: &Chat) -> Self {
//         NewTelegramChat {
//             id: chat.id.clone(),
//             kind: chat.kind.clone(),
//             title: chat.title.clone(),
//             username: chat.username.clone(),
//             first_name: chat.first_name.clone(),
//             last_name: chat.last_name.clone(),
//         }
//     }
// }
