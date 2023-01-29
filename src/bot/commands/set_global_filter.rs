use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/set_global_filter";

#[derive(TypedBuilder)]
pub struct SetGlobalFilter {
    message: Message,
    args: String,
}

impl SetGlobalFilter {
    pub fn run(&self) {
        self.execute(&self.message, &format!("{} {}", Self::command(), self.args));
    }

    fn set_global_template(&self, db_connection: &mut PgConnection) -> String {
        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        if self.args.is_empty() {
            return "Filter can not be empty".to_string();
        }

        let filter_words = match self.parse_filter(&self.args) {
            Err(message) => return message,
            Ok(words) => words,
        };

        match telegram::set_global_filter(db_connection, &chat, Some(filter_words.clone())) {
            Ok(_) => format!(
                "The global filter was updated:\n\n{}",
                filter_words.join(", ")
            ),
            Err(_) => "Failed to update the filter".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetGlobalFilter {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.set_global_template(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
