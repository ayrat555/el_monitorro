use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_global_filter";

#[derive(TypedBuilder)]
pub struct GetGlobalFilter {
    message: Message,
}

impl GetGlobalFilter {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    fn get_global_template(&self, db_connection: &mut PgConnection) -> String {
        match telegram::find_chat(db_connection, self.message.chat.id) {
            None => "You don't have the global filter set".to_string(),
            Some(chat) => match chat.filter_words {
                None => "You don't have the global filter set".to_string(),
                Some(filter_words) => {
                    format!("Your global filter is \n {}", filter_words.join(", "))
                }
            },
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetGlobalFilter {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.get_global_template(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
