use crate::bot::commands::help::send_message_params_builder;
use crate::bot::commands::help::set_help_keyboard;
use crate::bot::commands::help::set_subscribe_keyboard;
use crate::bot::commands::set_global_template::set_global_template_bold_keyboard;
use crate::bot::commands::set_global_template::set_global_template_create_link_keyboard;
use crate::bot::commands::set_global_template::set_global_template_italic_keyboard;
use crate::bot::commands::set_global_template::set_global_template_keyboard;
use crate::bot::commands::set_global_template::set_global_template_substring_keyboard;
use crate::bot::commands::set_template::select_feed_url;
use crate::bot::commands::set_template::set_template_keyboard;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds;
use crate::db::telegram;
use crate::db::telegram::NewTelegramChat;
use crate::db::telegram::NewTelegramSubscription;
use crate::models::Feed;
use crate::models::TelegramSubscription;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::CallbackQuery;
use frankenstein::Chat;
use frankenstein::ChatType;
use frankenstein::DeleteMessageParams;
use frankenstein::Message;
use frankenstein::TelegramApi;

// use async_trait::async_trait;

pub mod get_filter;
pub mod get_global_filter;
pub mod get_global_template;
pub mod get_template;
pub mod get_timezone;
pub mod help;
pub mod info;
pub mod list_subscriptions;
pub mod remove_filter;
pub mod remove_global_filter;
pub mod remove_global_template;
pub mod remove_template;
pub mod set_content_fields;
pub mod set_filter;
pub mod set_global_filter;
pub mod set_global_template;
pub mod set_template;
pub mod set_timezone;
pub mod start;
pub mod subscribe;
pub mod unknown_command;
pub mod unsubscribe;

const BOT_NAME: &str = "@sasaathulbot "; //replace with your bot name add a space after the name

impl From<Chat> for NewTelegramChat {
    fn from(chat: Chat) -> Self {
        let kind = match chat.type_field {
            ChatType::Private => "private",
            ChatType::Group => "group",
            ChatType::Supergroup => "supergroup",
            ChatType::Channel => "channel",
        };

        NewTelegramChat {
            id: chat.id as i64,
            kind: kind.to_string(),
            username: chat.username,
            first_name: chat.first_name,
            last_name: chat.last_name,
            title: chat.title,
        }
    }
}

pub trait Command {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        api: &Api,
    ) -> String;
   
    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        let db_connection: &mut PgConnection;
        let messages = message.text.as_ref().unwrap().to_string();
        let chat_id = message.chat.id;
        let response_message = messages.replace(BOT_NAME, "");
        let data = match self.fetch_db_connection(db_pool.clone()) {
            Ok(mut connection) => self.list_subscriptions(&mut *connection, message.clone()),
            Err(error_message) => "error fetching data".to_string(),
        };

        info!("{:?} wrote: {}", message.chat.id, response_message,);
        let text = self.response(db_pool.clone(), &message, &api);
        println!(
            "text in execute ================{}",
            message.text.as_ref().unwrap()
        );
        if messages == "/subscribe" {
            let send_message_params =
                send_message_params_builder(set_subscribe_keyboard(), chat_id, messages);
            api.send_message(&send_message_params).unwrap();
            // println!("nothing");
        } 
        // else if messages == "/set_global_template" {
        //     let send_message_params = set_global_template_keyboard(message);
        //     api.send_message(&send_message_params).unwrap();
        // } 
        else if messages == "/set_global_template substring" {
            let send_message_params = set_global_template_substring_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if messages == "/set_global_template italic" {
            let send_message_params = set_global_template_italic_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if messages == "create_link" {
            let send_message_params = set_global_template_create_link_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if messages == "/set_global_template bold" {
            let send_message_params = set_global_template_bold_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if messages == "/set_global_template create_link" {
            let send_message_params = set_global_template_create_link_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if messages == "/help" {
            let send_message_params = set_help_keyboard(chat_id);
            api.send_message(&send_message_params).unwrap();
        } else if messages == "/list_subscriptions" {
            self.reply_to_message(api, message, text);
        } else if messages == "/set_template" {
            let send_message_params = select_feed_url(message.clone(), data);
            //    let data = ListSubscriptions::execute(db_pool, api.clone(), message);
            api.send_message(&send_message_params).unwrap();
        } else {
            println!("excute recieved params {:?}", message);
            self.reply_to_message(api, message, text);
        }
    }

    fn execute_callback(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        api: Api,
        query: CallbackQuery,
    ) {
        let message = query.message.unwrap();
        let messages = query.data.as_ref().unwrap().to_string();
        let chat_id = query.from.id as i64;
        let messageid = message.message_id;
        let delete_message_params = DeleteMessageParams::builder()
            .chat_id(chat_id)
            .message_id(messageid)
            .build();
        let response_message = &messages.replace(BOT_NAME, "");
        let text = self.response(db_pool.clone(), &message, &api);
        info!("{:?} wrote: {}", message.chat.id, response_message);
        println!("text in execute callback ========{}", messages);
        // .as_ref().unwrap());

        if response_message == "/subscribe" {
            let send_message_params = send_message_params_builder(
                set_subscribe_keyboard(),
                chat_id,
                response_message.to_string(),
            );
            api.send_message(&send_message_params).unwrap();
        } else if response_message == "/set_global_template" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if response_message == "substring" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_substring_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if response_message == "italic" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_italic_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if response_message == "bold" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_bold_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if response_message == "create_link" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_global_template_create_link_keyboard(message);
            api.send_message(&send_message_params).unwrap();
        } else if response_message == "/help" {
            api.delete_message(&delete_message_params).unwrap();
            let send_message_params = set_help_keyboard(chat_id);
            api.send_message(&send_message_params).unwrap();
        } else if response_message == "/set_global_template {{italic bot_item_description }}" {
            // let send_message_params = set_help_keyboard(chat_id);
            // api.send_message(&send_message_params).unwrap();
            // GetGlobalTemplate::execute(db_pool, api, message);
            self.reply_to_callback(api, message, text);
        } else {
            // let send_message_params = set_template_keyboard(&message);
            // api.send_message(&send_message_params).unwrap();
            print!("text in message {}", text);
            self.reply_to_message(api, message, text);
        }
        // } else {
        //     self.reply_to_message(api, message, text);
        // }
    }
    // fn execute_callback_command(&self, db_pool: Pool<ConnectionManager<PgConnection>>,  api: Api, chat_id: i64,query: CallbackQuery,message: Message) {

    //     let messages = query.data.as_ref().unwrap().to_string();
    //     let response_message = messages.replace(BOT_NAME, "");
    //     let messageid=message.message_id;
    //     let text = self.response(db_pool.clone(), &message, &api);
    //     //  info!("{:?} wrote: ", response_message.clone());

    //     if response_message == "/set_global_template {{italic bot_item_description }}" {

    //         self.reply_to_message(api, );
    //     }

    //     else{
    //         // let send_message_params = set_template_keyboard(&message);
    //         // api.send_message(&send_message_params).unwrap();
    //          print!("nothing recieved")
    //         // self.reply_to_message(api, message, text);
    //     }
    //     // } else {
    //     //     self.reply_to_message(api, message, text);
    //     // }
    // }

    fn reply_to_message(&self, api: Api, message: Message, text: String) {
        if let Err(error) =
            api.reply_with_text_message(message.chat.id, text, Some(message.message_id))
        {
            error!("Failed to reply to update {:?} {:?}", error, message);
        }
    }
    fn reply_to_callback(&self, api: Api, query: Message, text: String) {
        if let Err(error) = api.reply_with_text_message(query.chat.id, text, Some(query.message_id))
        {
            error!("Failed to reply to update {:?} {:?}", error, query);
        }
    }

    fn command(&self) -> &str;

    fn parse_argument(&self, full_command: &str) -> String {
        let command = self.command();
        let handle = Config::telegram_bot_handle();
        let command_with_handle = format!("{}@{}", command, handle);

        if full_command.starts_with(&command_with_handle) {
            full_command
                .replace(&command_with_handle, "")
                .replace(BOT_NAME, "")
                .trim()
                .to_string()
        } else {
            full_command
                .replace(command, "")
                .replace(BOT_NAME, "")
                .trim()
                .to_string()
        }
    }

    fn fetch_db_connection(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, String> {
        match db_pool.get() {
            Ok(connection) => Ok(connection),
            Err(err) => {
                error!("Failed to fetch a connection from the pool {:?}", err);

                Err("Failed to process your command. Please contact @Ayrat555".to_string())
            }
        }
    }

    fn find_subscription(
        &self,
        db_connection: &mut PgConnection,
        chat_id: i64,
        feed_url: String,
    ) -> Result<TelegramSubscription, String> {
        let not_exists_error = Err("Subscription does not exist".to_string());
        let feed = self.find_feed(db_connection, feed_url)?;

        let chat = match telegram::find_chat(db_connection, chat_id) {
            Some(chat) => chat,
            None => return not_exists_error,
        };

        let telegram_subscription = NewTelegramSubscription {
            chat_id: chat.id,
            feed_id: feed.id,
        };

        match telegram::find_subscription(db_connection, telegram_subscription) {
            Some(subscription) => Ok(subscription),
            None => not_exists_error,
        }
    }

    fn find_feed(
        &self,
        db_connection: &mut PgConnection,
        feed_url: String,
    ) -> Result<Feed, String> {
        match feeds::find_by_link(db_connection, feed_url) {
            Some(feed) => Ok(feed),
            None => Err("Feed does not exist".to_string()),
        }
    }

    fn parse_filter(&self, params: &str) -> Result<Vec<String>, String> {
        let filter_words: Vec<String> =
            params.split(',').map(|s| s.trim().to_lowercase()).collect();

        if filter_words.len() > 7 {
            return Err("The number of filter words is limited by 7".to_string());
        }

        Ok(filter_words)
    }
    fn list_subscriptions(&self, db_connection: &mut PgConnection, message: Message) -> String {
        match telegram::find_feeds_by_chat_id(db_connection, message.chat.id) {
            Err(_) => "Couldn't fetch your subscriptions".to_string(),
            Ok(feeds) => {
                if feeds.is_empty() {
                    "You don't have any subscriptions".to_string()
                } else {
                    feeds
                        .into_iter()
                        .map(|feed| feed.link)
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            }
        }
    }
}
// pub fn get_chat_id()-> i64{
//     // let db_connection: &mut PgConnection;
//     // let other_chat_id: i64;
//     // let chat_not_exists_error = Err("Subscription does not exist".to_string());
//      let chat = match telegram::find_chat(db_connection, other_chat_id) {
//          Some(chat) => chat,
//          None => return 1 ,
//      };
//   return chat.id
//  }
