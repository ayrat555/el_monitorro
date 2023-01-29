use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/toggle_preview_enabled";

#[derive(TypedBuilder)]
pub struct TogglePreviewEnabled {
    message: Message,
}

impl TogglePreviewEnabled {
    pub fn run(&self) {
        self.execute(&self.message, Self::command());
    }

    fn toggle_preview_enabled(&self, db_connection: &mut PgConnection) -> String {
        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        match telegram::set_preview_enabled(db_connection, &chat, !chat.preview_enabled) {
            Ok(updated_chat) => {
                if updated_chat.preview_enabled {
                    "Previews are now enabled".to_string()
                } else {
                    "Previews are now disabled".to_string()
                }
            }

            Err(_) => "Failed to update the chat".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for TogglePreviewEnabled {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.toggle_preview_enabled(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
