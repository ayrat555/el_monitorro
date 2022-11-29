use super::Command;
use super::Message;
use crate::db::telegram;
use crate::deliver::render_template_example;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/set_template";
static CALLBACK: &str = "set_template";
static DEFAULT_TEMPLATE: &str = "set_default_template";
static SET_TEMPLATE_CREATE_LINK_DESCRIPTION: &str = "set__template_description";
static SET_TEMPLATE_CREATE_LINK_BOT_ITEM_NAME: &str = "set__template_item_name";
#[derive(TypedBuilder)]
pub struct SetTemplate {
    message: Message,
    args: String,
}

impl SetTemplate {
    pub fn run(&self) {
        self.execute(&self.message);
    }
    fn set_template(&self, db_connection: &mut PgConnection) -> String {
        let vec: Vec<&str> = self.args.splitn(2, ' ').collect();

        if vec.len() != 2 {
            return "Wrong number of parameters".to_string();
        }

        if vec[1].is_empty() {
            return "Template can not be empty".to_string();
        }

        let feed_url = vec[0].trim();

        let template = vec[1];

        let subscription =
            match self.find_subscription(db_connection, self.message.chat.id, feed_url) {
                Err(message) => return message,
                Ok(subscription) => subscription,
            };

        let example = match render_template_example(template) {
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

        match telegram::set_template(db_connection, &subscription, Some(template.to_string())) {
            Ok(_) => "The template was updated".to_string(),
            Err(_) => "Failed to update the template".to_string(),
        }
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    pub fn callback() -> &'static str {
        CALLBACK
    }

    pub fn default_template() -> &'static str {
        DEFAULT_TEMPLATE
    }

    pub fn create_link_description() -> &'static str {
        SET_TEMPLATE_CREATE_LINK_DESCRIPTION
    }

    pub fn create_link_item_name() -> &'static str {
        SET_TEMPLATE_CREATE_LINK_BOT_ITEM_NAME
    }
}

impl Command for SetTemplate {
    fn response(&self) -> String {
        match self.fetch_db_connection() {
            Ok(mut connection) => self.set_template(&mut connection),
            Err(error_message) => error_message,
        }
    }
}
