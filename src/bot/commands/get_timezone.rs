use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_timezone";

#[derive(TypedBuilder)]
pub struct GetTimezone {
    message: Message,
}

impl GetTimezone {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    fn get_timezone(&self, db_connection: &mut PgConnection) -> String {
        match telegram::find_chat(db_connection, self.message.chat.id) {
            None => "You don't have timezone set".to_string(),
            Some(chat) => match chat.utc_offset_minutes {
                None => "You don't have timezone set".to_string(),
                Some(value) => format!("Your timezone offset is {value} minutes"),
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetTimezone {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.get_timezone(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
