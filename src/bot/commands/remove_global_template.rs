use super::Command;
use super::Message;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/remove_global_template";

#[derive(TypedBuilder)]
pub struct RemoveGlobalTemplate {
    message: Message,
}

impl RemoveGlobalTemplate {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    fn remove_global_template(&self, db_connection: &mut PgConnection) -> String {
        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        match telegram::set_global_template(db_connection, &chat, None) {
            Ok(_) => "The global template was removed".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for RemoveGlobalTemplate {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.remove_global_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
