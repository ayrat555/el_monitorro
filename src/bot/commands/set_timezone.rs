use super::Command;
use super::Message;
use super::Response;
use crate::db::telegram;
use diesel::PgConnection;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/set_timezone";

#[derive(TypedBuilder)]
pub struct SetTimezone {
    message: Message,
    args: String,
}

impl SetTimezone {
    pub fn run(&self) {
        self.execute(&self.message, &format!("{} {}", Self::command(), self.args));
    }

    fn set_timezone(&self, db_connection: &mut PgConnection) -> String {
        match self.update_timezone(db_connection) {
            Ok(_) => "Your timezone was updated".to_string(),
            Err(err_string) => err_string.to_string(),
        }
    }

    fn update_timezone(&self, db_connection: &mut PgConnection) -> Result<(), &str> {
        let offset = self.validate_offset()?;

        match telegram::find_chat(db_connection, self.message.chat.id) {
            None => Err(
                "You'll be able to set your timezone only after you'll have at least one subscription",
            ),
            Some(chat) => match telegram::set_utc_offset_minutes(db_connection, &chat, offset) {
                Ok(_) => Ok(()),
                Err(_) => Err("Failed to set your timezone"),
            },
        }
    }

    fn validate_offset(&self) -> Result<i32, &'static str> {
        let offset = match self.args.parse::<i32>() {
            Ok(result) => result,
            Err(_) => return Err("The value is not a number"),
        };

        if offset % 30 != 0 {
            return Err("Offset must be divisible by 30");
        }

        if !(-720..=840).contains(&offset) {
            return Err("Offset must be >= -720 (UTC -12) and <= 840 (UTC +14)");
        }

        Ok(offset)
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for SetTimezone {
    fn response(&self) -> Response {
        let response = match self.fetch_db_connection() {
            Ok(mut connection) => self.set_timezone(&mut connection),
            Err(error_message) => error_message,
        };

        Response::Simple(response)
    }
}
