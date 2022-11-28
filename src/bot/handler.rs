use super::commands::list_subscriptions_inline_keyboard::ListSubscriptionsInlineKeyboard;
use super::commands::set_global_template_inline_keyboard::SetGlobalTemplateInlineKeyboard;
use super::commands::set_template_inline_keyboard::SetTemplateInlineKeyboard;
use super::commands::BotCommand;
use super::commands::GetFilter;
use super::commands::GetGlobalFilter;
use super::commands::GetGlobalTemplate;
use super::commands::GetTemplate;
use super::commands::GetTimezone;
use super::commands::Help;
use super::commands::Info;
use super::commands::ListSubscriptions;
use super::commands::RemoveFilter;
use super::commands::RemoveGlobalFilter;
use super::commands::RemoveGlobalTemplate;
use super::commands::RemoveTemplate;
use super::commands::SetContentFields;
use super::commands::SetFilter;
use super::commands::SetGlobalFilter;
use super::commands::SetGlobalTemplate;
use super::commands::SetTemplate;
use super::commands::SetTimezone;
use super::commands::Start;
use super::commands::Subscribe;
use super::commands::UnknownCommand;
use super::commands::Unsubscribe;
use super::telegram_client::Api;

use crate::bot::commands::parse_args;
use crate::config::Config;
use crate::db::feeds::find;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::DeleteMessageParams;
use frankenstein::TelegramApi;
use frankenstein::Update;
use frankenstein::UpdateContent;
use regex::Regex;
use std::str::FromStr;
use std::thread;

const DEFAULT_TEMPLATE: &str = "{{bot_feed_name}}\n\n{{bot_item_name}}\n\n{{bot_item_description}}\n\n{{bot_date}}\n\n{{bot_item_link}}\n\n";

#[derive(Debug)]
enum CallbackDatas {
    ListSubscriptions,
    CallbackListSubscriptions(Option<String>, String),
    GetFilter(String),
    GetTemplate(String, String),
    SetTemplate(String, String),
    CallbackSetTemplateCreateLinkDescription(String),
    CallbackSetTemplateCreateLinkBotItemName(String),
    CallbackSetTemplate(String, String),
    CallbackSubstring(String),
    CallbackItalic,
    CallbackBold,
    CallbackCreateLink(String),
    CallbackSetDefaulTemplate(String),
    RemoveTemplate(String, String),
    RemoveFilter(String, String),
    SetGlobalTemplate,
    CallbackGlobalItalic,
    CallbackGlobalBold,
    CallbackGlobalCreateLink,
    CallbackGlobalSubstring,
    CallbackGlobalDefaultTemplate,
    Unsubscribe(String, String),
    CallbackGlobalTemplateCreateLinkDescription,
    CallbackGlobalTemplateCreateLinkBotItemName,
    CallbackUnsubscribe(Option<String>, String),
    CallbackBackToMenu,
    UnknownCommand(String),
}
fn parse_int_from_string(command: &str) -> Option<std::string::String> {
    let re = Regex::new(
        r"(?x)
            (?P<name>\d+)  # the name
        ",
    )
    .unwrap();
    let data: Option<String> = re.captures(command).map(|s| s["name"].to_string());
    match data {
        Some(s) => Some(s),
        None => Some("no integer".to_string()),
    }
}

pub fn get_feed_url_by_id(db_pool: Pool<ConnectionManager<PgConnection>>, data: String) -> String {
    match data.is_empty() {
        false => {
            let feedid: i64 = data.parse().unwrap();
            match fetch_db_connection(db_pool) {
                Ok(mut connection) => {
                    let feeds = find(&mut connection, feedid).unwrap();
                    let data = feeds;
                    data.link
                }
                Err(_error_message) => "error fetching message".to_string(),
            }
        }
        true => "you dont have any subscriptions".to_string(),
    }
}
pub fn fetch_db_connection(
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

impl FromStr for CallbackDatas {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let st = s.to_string();

        let db_pool = crate::db::pool().clone();

        let result = match st {
            st if st.starts_with(ListSubscriptions::command()) => CallbackDatas::ListSubscriptions,
            st if st.starts_with(ListSubscriptions::callback()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                if feedid == "no integer" {
                    let feed_url = "".to_string();
                    CallbackDatas::CallbackListSubscriptions(Some(feedid), feed_url)
                } else {
                    let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                    CallbackDatas::CallbackListSubscriptions(Some(feedid), feed_url)
                }
            }
            st if st.starts_with(GetFilter::command()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid);
                CallbackDatas::GetFilter(feed_url)
            }
            st if st.starts_with(GetTemplate::command()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                CallbackDatas::GetTemplate(feedid, feed_url)
            }
            st if st.starts_with(SetTemplate::command()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                CallbackDatas::SetTemplate(feedid, feed_url)
            }
            st if st.starts_with(SetTemplate::callback()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                if feedid == "no integer" {
                    let feed_url = "".to_string();
                    CallbackDatas::CallbackSetTemplate(feed_url, feedid)
                } else {
                    let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                    CallbackDatas::CallbackSetTemplate(feed_url, feedid)
                }
            }
            st if st.starts_with(SetTemplate::create_link_description()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid);
                CallbackDatas::CallbackSetTemplateCreateLinkDescription(feed_url)
            }
            st if st.starts_with(SetTemplate::create_link_item_name()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid);
                CallbackDatas::CallbackSetTemplateCreateLinkBotItemName(feed_url)
            }
            st if st.starts_with(SetTemplateInlineKeyboard::substring()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid);
                CallbackDatas::CallbackSubstring(feed_url)
            }
            st if st.starts_with(SetTemplateInlineKeyboard::italic()) => {
                CallbackDatas::CallbackItalic
            }
            st if st.starts_with(SetTemplateInlineKeyboard::bold()) => CallbackDatas::CallbackBold,
            st if st.starts_with(SetTemplateInlineKeyboard::create_link()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid);
                CallbackDatas::CallbackCreateLink(feed_url)
            }
            st if st.starts_with(SetTemplate::default_template()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid);
                CallbackDatas::CallbackSetDefaulTemplate(feed_url)
            }

            st if st.starts_with(RemoveTemplate::command()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                CallbackDatas::RemoveTemplate(feedid, feed_url)
            }
            st if st.starts_with(RemoveFilter::command()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                CallbackDatas::RemoveFilter(feedid, feed_url)
            }
            st if st.starts_with(SetGlobalTemplate::command()) => CallbackDatas::SetGlobalTemplate,
            st if st.starts_with(SetGlobalTemplate::create_link_description()) => {
                CallbackDatas::CallbackGlobalTemplateCreateLinkDescription
            }
            st if st.starts_with(SetGlobalTemplate::create_link_item_name()) => {
                CallbackDatas::CallbackGlobalTemplateCreateLinkBotItemName
            }
            st if st.starts_with(SetGlobalTemplateInlineKeyboard::italic()) => {
                CallbackDatas::CallbackGlobalItalic
            }
            st if st.starts_with(SetGlobalTemplateInlineKeyboard::bold()) => {
                CallbackDatas::CallbackGlobalBold
            }
            st if st.starts_with(SetGlobalTemplateInlineKeyboard::create_link()) => {
                CallbackDatas::CallbackGlobalCreateLink
            }
            st if st.starts_with(SetGlobalTemplateInlineKeyboard::substring()) => {
                CallbackDatas::CallbackGlobalSubstring
            }
            st if st.starts_with(SetGlobalTemplateInlineKeyboard::default_template()) => {
                CallbackDatas::CallbackGlobalDefaultTemplate
            }
            st if st.starts_with(Unsubscribe::command()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                CallbackDatas::Unsubscribe(feedid, feed_url)
            }
            st if st.starts_with(Unsubscribe::callback()) => {
                let feedid = parse_int_from_string(&st).unwrap();
                if feedid == "no integer" {
                    let feed_url = "".to_string();
                    CallbackDatas::CallbackUnsubscribe(Some(feedid), feed_url)
                } else {
                    let feed_url = get_feed_url_by_id(db_pool, feedid.clone());
                    CallbackDatas::CallbackUnsubscribe(Some(feedid), feed_url)
                }
            }
            st if st.starts_with(ListSubscriptionsInlineKeyboard::back_to_menu()) => {
                CallbackDatas::CallbackBackToMenu
            }
            _ => CallbackDatas::UnknownCommand(st),
        };
        Ok(result)
    }
}

pub struct Handler {}

impl Handler {
    pub fn start() {
        // maybe Api can be share also
        let mut api = Api::default();
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(Config::commands_db_pool_number() as usize)
            .build()
            .unwrap();

        log::info!("Starting the El Monitorro bot");

        let interval = std::time::Duration::from_secs(1);
        loop {
            while let Some(update) = api.next_update() {
                let db_pool = crate::db::pool().clone();
                let tg_api = api.clone();

                match update.content {
                    UpdateContent::Message(_) => {
                        thread_pool.spawn(move || {
                            Self::process_message_or_channel_post(db_pool, tg_api, update)
                        });
                    }
                    UpdateContent::ChannelPost(_) => {
                        thread_pool.spawn(move || {
                            Self::process_message_or_channel_post(db_pool, tg_api, update)
                        });
                    }
                    UpdateContent::CallbackQuery(_) => {
                        thread_pool
                            .spawn(move || Self::process_callback_query(db_pool, tg_api, update));
                    }
                    _ => return,
                }
            }

            thread::sleep(interval);
        }
    }

    fn process_message_or_channel_post(
        db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
        api: Api,
        update: Update,
    ) {
        let message = match update.content {
            UpdateContent::Message(message) => message,
            UpdateContent::ChannelPost(channel_post) => channel_post,
            _ => return,
        };

        if let Some(owner_id) = Self::owner_telegram_id() {
            if message.from.is_none() {
                return;
            }

            if message.from.as_ref().unwrap().id as i64 != owner_id {
                return;
            }
        }

        let text = message.text.clone();

        if text.is_none() {
            return;
        }
        let bot_name = Config::telegram_bot_handle();
        let binding = text.unwrap().replace(&bot_name, "");
        let commands = binding.as_str();

        let command = BotCommand::from_str(commands).unwrap();

        match command {
            BotCommand::Subscribe(args) => Subscribe::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::Help => Help::builder()
                ._message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::Unsubscribe(args) => Unsubscribe::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::ListSubscriptions => ListSubscriptions::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::Start => Start::builder()
                ._message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::SetTimezone(args) => SetTimezone::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::GetTimezone => GetTimezone::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::SetFilter(args) => SetFilter::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::GetFilter(args) => GetFilter::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::RemoveFilter(args) => RemoveFilter::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::SetTemplate(args) => SetTemplate::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::GetTemplate(args) => GetTemplate::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::RemoveTemplate(args) => RemoveTemplate::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::SetGlobalTemplate(args) => SetGlobalTemplate::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::RemoveGlobalTemplate => RemoveGlobalTemplate::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::GetGlobalTemplate => GetGlobalTemplate::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::SetGlobalFilter(args) => SetGlobalFilter::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::GetGlobalFilter => GetGlobalFilter::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::RemoveGlobalFilter => RemoveGlobalFilter::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::Info => Info::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),

            BotCommand::SetContentFields(args) => SetContentFields::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),

            BotCommand::UnknownCommand(args) => UnknownCommand::builder()
                .message(message.clone())
                .args(args)
                .build()
                .run(db_pool, api, message),
        };
    }

    fn owner_telegram_id() -> Option<i64> {
        Config::owner_telegram_id()
    }

    fn process_callback_query(
        db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
        api: Api,
        update: Update,
    ) {
        let bot_name = Config::telegram_bot_handle();
        let query = match update.content {
            UpdateContent::CallbackQuery(callback_query) => callback_query,
            _ => return,
        };

        let mut message = query.message.unwrap();
        let messageid = message.message_id;
        let chatid = message.chat.id;
        let text = query.data;
        let delete_message_params = DeleteMessageParams::builder()
            .chat_id(chatid)
            .message_id(messageid)
            .build();
        if text.is_none() {
            return;
        }

        let commands = &text.unwrap();
        let data = &commands.replace(&bot_name, "");
        message.text = Some(data.clone());

        let command = CallbackDatas::from_str(commands).unwrap();

        match command {
            CallbackDatas::ListSubscriptions => ListSubscriptions::builder()
                .message(message.clone())
                .build()
                .run(db_pool, api, message),
            CallbackDatas::CallbackListSubscriptions(feedid, feed_url) => {
                match feed_url.is_empty() {
                    true => ListSubscriptions::builder()
                        .message(message.clone())
                        .build()
                        .run(db_pool, api, message),
                    false => {
                        api.delete_message(&delete_message_params).unwrap();
                        let send_message_params =
                            ListSubscriptionsInlineKeyboard::set_list_subcriptions_menu_keyboard(
                                message,
                                feedid.unwrap(),
                                feed_url,
                            );
                        api.send_message(&send_message_params).unwrap();
                    }
                }
            }
            CallbackDatas::GetFilter(feed_url) => {
                message.text = Some(format!("/get_filter {}", feed_url));

                GetFilter::builder()
                    .message(message.clone())
                    .args(feed_url)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::GetTemplate(feedid, feed_url) => {
                let text = &commands.replace(&feedid, &feed_url);
                message.text = Some(text.trim().to_string());
                let args = parse_args(GetTemplate::command(), text);

                GetTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::SetTemplate(feedid, feed_url) => {
                let text = &commands.replace(&feedid, &feed_url);
                let args = parse_args(SetTemplate::command(), text);
                message.text = Some(text.trim().to_string());

                SetTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::CallbackSetTemplateCreateLinkDescription(feed_url) => {
                message.text = Some(format!(
                    "/set_template {} {{create_link bot_item_description bot_item_link}}",
                    feed_url
                ));
                let args = parse_args(SetTemplate::command(), &message.clone().text.unwrap());

                SetTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::CallbackSetTemplateCreateLinkBotItemName(feed_url) => {
                message.text = Some(format!(
                    "/set_template {} {{create_link bot_item_description bot_item_link}}",
                    feed_url
                ));

                let args = parse_args(SetTemplate::command(), &message.clone().text.unwrap());

                SetTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::CallbackSetTemplate(feed_url, feedid) => match feed_url.is_empty() {
                true => {
                    let args = "".to_string();
                    SetTemplate::builder()
                        .message(message.clone())
                        .args(args)
                        .build()
                        .run(db_pool, api, message)
                }
                false => {
                    let send_message_params =
                        SetTemplateInlineKeyboard::set_template_menu_keyboard(message, feedid);
                    api.send_message(&send_message_params).unwrap();
                }
            },
            CallbackDatas::CallbackSubstring(feed_url) => {
                api.delete_message(&delete_message_params).unwrap();
                let data = commands.replace("substring", "");

                let send_message_params =
                    SetTemplateInlineKeyboard::set_template_substring_keyboard(
                        message, data, feed_url,
                    );
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackItalic => {
                api.delete_message(&delete_message_params).unwrap();
                let data = &commands.replace("italic", "");

                let send_message_params = SetTemplateInlineKeyboard::set_template_italic_keyboard(
                    message,
                    data.to_string(),
                );
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackBold => {
                api.delete_message(&delete_message_params).unwrap();
                let data = &commands.replace("bold", "");

                let send_message_params = SetTemplateInlineKeyboard::set_template_bold_keyboard(
                    message,
                    data.to_string(),
                );
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackCreateLink(feed_url) => {
                api.delete_message(&delete_message_params).unwrap();
                let data = parse_int_from_string(commands).unwrap();

                let send_message_params =
                    SetTemplateInlineKeyboard::set_template_create_link_keyboard(
                        message, data, feed_url,
                    );
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackSetDefaulTemplate(feed_url) => {
                let text =
                    Some(format!("/set_template {} {}", feed_url, DEFAULT_TEMPLATE)).unwrap();

                let args = parse_args(SetTemplate::command(), &text);
                message.text = Some(format!("/set_template {} {}", feed_url, DEFAULT_TEMPLATE));

                SetTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::RemoveTemplate(feedid, feed_url) => {
                let text = &commands.replace(&feedid, &feed_url);
                message.text = Some(text.trim().to_string());
                let args = parse_args(RemoveTemplate::command(), text);

                RemoveTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::RemoveFilter(feedid, feed_url) => {
                let text = &commands.replace(&feedid, &feed_url);
                message.text = Some(text.trim().to_string());
                let args = parse_args(RemoveFilter::command(), text);

                RemoveFilter::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::SetGlobalTemplate => {
                let args = parse_args(SetGlobalTemplate::command(), commands);

                SetGlobalTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::CallbackGlobalTemplateCreateLinkDescription => {
                message.text = Some(
                    "/set_global_template {{create_link bot_item_description bot_item_link}}"
                        .to_string(),
                );

                let args = "{{create_link bot_item_description bot_item_link}}".to_string();

                SetGlobalTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::CallbackGlobalTemplateCreateLinkBotItemName => {
                message.text = Some(
                    "/set_global_template {{create_link bot_item_name bot_item_link}}".to_string(),
                );

                let args = "{{create_link bot_item_name bot_item_link}}".to_string();

                SetGlobalTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::CallbackGlobalItalic => {
                api.delete_message(&delete_message_params).unwrap();
                let send_message_params =
                    SetGlobalTemplateInlineKeyboard::set_global_template_italic_keyboard(message);
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackGlobalBold => {
                api.delete_message(&delete_message_params).unwrap();
                let send_message_params =
                    SetGlobalTemplateInlineKeyboard::set_global_template_bold_keyboard(message);
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackGlobalCreateLink => {
                api.delete_message(&delete_message_params).unwrap();
                let send_message_params =
                    SetGlobalTemplateInlineKeyboard::set_global_template_create_link_keyboard(
                        message,
                    );
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackGlobalSubstring => {
                api.delete_message(&delete_message_params).unwrap();
                let send_message_params =
                    SetGlobalTemplateInlineKeyboard::set_global_template_substring_keyboard(
                        message,
                    );
                api.send_message(&send_message_params).unwrap();
            }
            CallbackDatas::CallbackGlobalDefaultTemplate => {
                api.delete_message(&delete_message_params).unwrap();
                message.text = Some(format!("/set_global_template {}", DEFAULT_TEMPLATE));
                let args = DEFAULT_TEMPLATE.to_string();

                SetGlobalTemplate::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::Unsubscribe(feedid, feed_url) => {
                let text = &commands.replace(&feedid, &feed_url);
                message.text = Some(text.trim().to_string());
                let args = parse_args(Unsubscribe::command(), text);

                Unsubscribe::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
            CallbackDatas::CallbackUnsubscribe(_feedid, feed_url) => match feed_url.is_empty() {
                true => {
                    let args = feed_url;

                    Unsubscribe::builder()
                        .message(message.clone())
                        .args(args)
                        .build()
                        .run(db_pool, api, message)
                }
                false => {
                    message.text = Some(format!("/unsubscribe {}", feed_url));
                    let args = feed_url;

                    Unsubscribe::builder()
                        .message(message.clone())
                        .args(args)
                        .build()
                        .run(db_pool, api, message)
                }
            },
            CallbackDatas::CallbackBackToMenu => {
                api.delete_message(&delete_message_params).unwrap();
                let send_message_params =
                    SetGlobalTemplateInlineKeyboard::set_global_template_keyboard(message);
                api.send_message(&send_message_params).unwrap();
            }
            _ => {
                let args = parse_args(UnknownCommand::command(), commands);

                UnknownCommand::builder()
                    .message(message.clone())
                    .args(args)
                    .build()
                    .run(db_pool, api, message)
            }
        }
    }
}
