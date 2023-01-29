use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_preview_enabled";

#[derive(TypedBuilder)]
pub struct GetPreviewEnabled {
    message: Message,
}

impl GetPreviewEnabled {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    fn get_preview_enabled(&self, db_connection: &mut PgConnection) -> String {
        match telegram::find_chat(db_connection, self.message.chat.id) {
            None => "You don't have any subscriptions".to_string(),
            Some(chat) => {
                if chat.preview_enabled {
                    "Previews are enabled in this chat".to_string()
                } else {
                    "Previews are disabled in this chat".to_string()
                }
            }
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetPreviewEnabled {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.get_preview_enabled(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
