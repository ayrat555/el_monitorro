use super::Command;
use super::Message;
use super::Response;
use crate::bot::SimpleMessageParams;
use crate::db::telegram;
use crate::deliver::render_template_example;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/set_template";

#[derive(TypedBuilder)]
pub struct SetTemplate {
    message: Message,
    args: String,
}

impl SetTemplate {
    pub fn run(&self) {
        self.execute(&self.message, &format!("{} {}", Self::command(), self.args));
    }

    fn set_template(&self, db_connection: &mut PgConnection) -> String {
        let vec: Vec<&str> = self.args.splitn(2, ' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Template can not be empty".to_string();
        }

        let feed_url = vec[0];
        let template = vec[1];

        let chat = match telegram::find_chat(db_connection, self.message.chat.id) {
            Some(chat) => chat,
            None => return "You don't have any subcriptions".to_string(),
        };

        let subscription =
            match self.find_subscription(db_connection, self.message.chat.id, feed_url) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        let example = match render_template_example(template) {
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

        match telegram::set_template(db_connection, &subscription, Some(template.to_string())) {
            Ok(_) => "The template was updated".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetTemplate {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.set_template(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
