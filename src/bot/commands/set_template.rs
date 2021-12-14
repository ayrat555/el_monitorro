use super::Command;
use super::Message;
use super::Template;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/set_template";

pub struct SetTemplate {}

impl SetTemplate {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn set_template(
        &self,
        db_connection: &PgConnection,
        message: &Message,
        params: String,
    ) -> String {
        let vec: Vec<&str> = params.split(' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Template can not be empty".to_string();
        }

        let subscription =
            match self.find_subscription(db_connection, message.chat.id, vec[0].to_string()) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        match self.parse_template_and_send_example(vec[1].to_string()) {
            Ok((template, example)) => {
                match telegram::set_template(db_connection, &subscription, template) {
                    Ok(_) => format!(
                        "The template was updated. Your messages will look like:\n\n{}",
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

impl Command for SetTemplate {
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(&text);
                self.set_template(&connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}

impl Template for SetTemplate {}

#[cfg(test)]
mod set_template_tests {
    use super::SetTemplate;
    use crate::bot::commands::Template;

    #[test]
    fn parse_template() {
        let template = "bot_feed_namehellobot_new_linebot_datebot_space";
        let result = SetTemplate {}.parse_template(template);

        assert_eq!(result, "{{bot_feed_name}}hello\n{{bot_date}} ".to_string());
    }
}
