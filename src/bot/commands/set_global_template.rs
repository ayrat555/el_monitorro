use super::Command;
use super::Message;
use super::Response;
use crate::bot::SimpleMessageParams;
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
        self.execute(&self.message, &format!("{} {}", Self::command(), self.args));
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
            Ok(example) => format!("Your messages will look like:\n\n{example}"),
            Err(_) => return "The template is invalid".to_string(),
        };

        let message_params = SimpleMessageParams::builder()
            .message(example)
            .chat_id(self.message.chat.id)
            .preview_enabled(chat.preview_enabled)
            .build();

        if self.api().reply_with_text_message(&message_params).is_err() {
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
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.set_global_template(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
