use super::Command;
use super::Message;
use super::Response;
use super::ShowFeedKeyboard;
use diesel::PgConnection;
use frankenstein::SendMessageParams;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/get_filter";

#[derive(TypedBuilder)]
pub struct GetFilter {
    message: Message,
    args: String,
    callback: bool,
}

impl GetFilter {
    pub fn run(&self) {
        self.execute(&self.message, &format!("{} {}", Self::command(), self.args));
    }

    fn get_filter(&self, db_connection: &mut PgConnection) -> Response {
        let subscription =
            match self.find_subscription(db_connection, self.message.chat.id, &self.args) {
                Ok(subscription) => subscription,
                Err(error) => return Response::Simple(error),
            };

        let response = match subscription.filter_words {
            None => "You did not set a filter for this subcription".to_string(),
            Some(filter_words) => filter_words.join(", "),
        };

        if self.callback {
            self.simple_keyboard(
                response,
                format!(
                    "{} {}",
                    ShowFeedKeyboard::command(),
                    subscription.external_id
                ),
                &self.message,
            )
        } else {
            Response::Simple(response)
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for GetFilter {
    fn response(&self) -> Response {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.get_filter(&mut connection),

            Err(error_message) => Response::Simple(error_message),
        }
    }

    fn send_message(&self, send_message_params: SendMessageParams) {
        self.send_message_and_remove(send_message_params, &self.message);
    }
}
