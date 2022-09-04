use super::unknown_command::UnknownCommand;
use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::config::Config;
use crate::db::feeds;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/set_content_fields";
static ALLOWED_CONTENT_FIELDS: [&str; 6] = [
    "link",
    "title",
    "publication_date",
    "guid",
    "description",
    "author",
];

pub struct SetContentFields {}

impl SetContentFields {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    pub fn set_content_fields(&self, db_connection: &mut PgConnection, params: String) -> String {
        let vec: Vec<&str> = params.split(' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Filter can not be empty".to_string();
        }

        let feed = match self.find_feed(db_connection, vec[0].to_string()) {
            Err(message) => return message,
            Ok(feed) => feed,
        };

        let content_fields: Vec<String> = vec[1]
            .split(',')
            .map(|field| field.trim().to_lowercase())
            .filter(|field| ALLOWED_CONTENT_FIELDS.contains(&field.as_str()))
            .collect();

        if content_fields.is_empty() {
            return "Invalid content fields".to_string();
        }

        match feeds::set_content_fields(db_connection, &feed, content_fields.clone()) {
            Ok(_) => format!(
                "Content fields were updated:\n\n{}",
                content_fields.join(", ")
            ),
            Err(_) => "Failed to update the content fields".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetContentFields {
    fn execute(&self, db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        match Config::admin_telegram_id() {
            None => UnknownCommand::execute(db_pool, api, message),
            Some(id) => {
                if id == message.chat.id {
                    info!(
                        "{:?} wrote: {}",
                        message.chat.id,
                        message.text.as_ref().unwrap()
                    );

                    let text = self.response(db_pool, &message, &api);

                    self.reply_to_message(api, message, text)
                } else {
                    UnknownCommand::execute(db_pool, api, message)
                }
            }
        }
    }

    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        _api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(mut connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(text);

                self.set_content_fields(&mut connection, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
