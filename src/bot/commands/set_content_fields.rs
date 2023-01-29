use super::unknown_command::UnknownCommand;
use super::Command;
use super::Message;
use super::Response;
use crate::config::Config;
use crate::db::feeds;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/set_content_fields";
static ALLOWED_CONTENT_FIELDS: [&str; 6] = [
    "link",
    "title",
    "publication_date",
    "guid",
    "description",
    "author",
];

#[derive(TypedBuilder)]
pub struct SetContentFields {
    message: Message,
    args: String,
}

impl SetContentFields {
    pub fn run(&self) {
        self.execute(&self.message, &format!("{} {}", Self::command(), self.args));
    }

    pub fn set_content_fields(&self, db_connection: &mut PgConnection) -> String {
        let vec: Vec<&str> = self.args.split(' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Filter can not be empty".to_string();
        }

        let feed = match self.find_feed(db_connection, vec[0]) {
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

    fn unknown_command(&self) {
        UnknownCommand::builder()
            .message(self.message.clone())
            .args(self.message.text.clone().unwrap())
            .build()
            .run();
    }
}

impl Command for SetContentFields {
    fn execute(&self, message: &Message, command: &str) {
        match Config::admin_telegram_id() {
            None => self.unknown_command(),
            Some(id) => {
                if id == message.chat.id {
                    info!("{:?} wrote: {}", message.chat.id, command);

                    if let Response::Simple(text) = self.response() {
                        self.reply_to_message(message, text);
                    }
                } else {
                    self.unknown_command()
                }
            }
        }
    }

    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.set_content_fields(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
