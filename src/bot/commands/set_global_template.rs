use super::Command;
use super::Message;
use super::Template;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/set_global_template";

pub struct SetGlobalTemplate {}

impl SetGlobalTemplate {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn set_global_template(
        &self,
        db_connection: &PgConnection,
        message: &Message,
        template: String,
    ) -> String {
        if template.is_empty() {
            return "Template can not be empty".to_string();
        }

        let chat = match telegram::find_chat(db_connection, message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        match self.parse_template_and_send_example(template) {
            Ok((template, example)) => {
                match telegram::set_global_template(db_connection, &chat, template) {
                    Ok(_) => format!(
                        "The global template was updated. Your messages will look like:\n\n{}",
                        example
                    ),
                    Err(_) => "Failed to update the template".to_string(),
                }
            }

            Err(error) => error,
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetGlobalTemplate {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(text);
                self.set_global_template(&connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}

impl Template for SetGlobalTemplate {}
