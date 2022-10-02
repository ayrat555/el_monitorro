use super::Command;
use super::Message;
use crate::db::telegram;
use crate::deliver::render_template_example;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/set_global_template";

#[derive(TypedBuilder)]
pub struct SetGlobalTemplate {
    message: Message,
    args: String,
}

impl SetGlobalTemplate {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    fn set_global_template(&self, db_connection: &mut PgConnection) -> String {
        if self.args.is_empty() {
            return "Template can not be empty".to_string();
        }

        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        let example = match render_template_example(&self.args) {
            Ok(example) => format!("Your messages will look like:\n\n{}", example),
            Err(_) => return "The template is invalid".to_string(),
        };

        if self
            .api()
            .send_text_message(self.message.chat.id, example)
            .is_err()
        {
            return "The template is invalid".to_string();
        }

        match telegram::set_global_template(db_connection, &chat, Some(self.args.clone())) {
            Ok(_) => "The global template was updated".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetGlobalTemplate {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.set_global_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
