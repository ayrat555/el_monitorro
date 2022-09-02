use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use crate::db::telegram;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

static COMMAND: &str = "/set_timezone";

pub struct SetTimezone {}

impl SetTimezone {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    fn set_timezone(
        &self,
        db_connection: &mut PgConnection,
        message: &Message,
        data: String,
    ) -> String {
        match self.update_timezone(db_connection, message, data) {
            Ok(_) => "Your timezone was updated".to_string(),
            Err(err_string) => err_string.to_string(),
        }
    }

    fn update_timezone(
        &self,
        db_connection: &mut PgConnection,
        message: &Message,
        data: String,
    ) -> Result<(), &str> {
        let offset = self.validate_offset(data)?;

        match telegram::find_chat(db_connection, message.chat.id) {
            None => Err(
                "You'll be able to set your timezone only after you'll have at least one subscription",
            ),
            Some(chat) => match telegram::set_utc_offset_minutes(db_connection, &chat, offset) {
                Ok(_) => Ok(()),
                Err(_) => Err("Failed to set your timezone"),
            },
        }
    }

    fn validate_offset(&self, offset_string: String) -> Result<i32, &'static str> {
        let offset = match offset_string.parse::<i32>() {
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
    fn response(
        &self,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        message: &Message,
        _api: &Api,
    ) -> String {
        match self.fetch_db_connection(db_pool) {
            Ok(mut connection) => {
                let text = message.text.as_ref().unwrap();
                let argument = self.parse_argument(text);
                self.set_timezone(&mut connection, message, argument)
            }
            Err(error_message) => error_message,
        }
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
