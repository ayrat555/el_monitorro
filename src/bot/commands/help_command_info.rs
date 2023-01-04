use super::help::HelpCommand;
use super::Command;
use super::Help;
use super::Response;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::PgConnection;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use std::str::FromStr;
use typed_builder::TypedBuilder;

static START: &str = "/start - show the description of the bot and its contact information";
static SUBSCRIBE: &str = "/subscribe url - subscribe to a feed";
static UNSUBSCRIBE: &str = "/unsubscribe url - unsubscribe from a feed";
static HELP: &str = "/help - show available commands";
static SET_TIMEZONE: &str = "set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, your offset is equal to 60 x 10 = 600";
static GET_TIMEZONE: &str = "/get_timezone - get your timezone";
static GET_TEMPLATE: &str = "/get_template feed_url - get the template for the subscription";
static SET_TEMPLATE: &str =
    "/set_template url template - set a template for all received feed items for the specified subscription. All new updates will be converted to the format defined by this subscription. Supported fields you can use for templates:\n\
     - bot_feed_name - name of the feed\n\
     - bot_feed_link - url of the feed\n\
     - bot_item_name - name of the item\n\
     - bot_item_link - url of the item\n\
     - bot_item_description - description of the item\n\
     - bot_date - publication date of the feed\n\
     Example: /set_template https://www.badykov.com/feed.xml {{bot_feed_name}}\n\n\n{{bot_item_name}}\n\n\n{{bot_date}}\n\n\n{{bot_item_link}}\n\n\
     Also, there are some helpers for templates:\n\n\
     - `substring` helper that can be used to limit the number of characters. For example, {{substring bot_item_description 100}}\n\
     - `create_link` helper. This helper creates an html link. For example, {{create_link bot_item_name bot_item_link}} or {{create_link \"custom_name\" bot_item_link}}\n\
     - `italic` helper. Usage: {{italic bot_item_description}}\n\
     - `bold` helper. Usage:  {{bold bot_item_name}}\n\n";

static REMOVE_TEMPLATE: &str =
    "/remove_template feed_url - remove the template for the subscription";
static SET_GLOBAL_TEMPLATE: &str =  "/set_global_template - set the global template. This template will be used for all subscriptions. If the subscription has its own template, it will be used instead. See /set_template for available fields.";
static GET_GLOBAL_TEMPLATE: &str = "/get_global_template - get global template";
static REMOVE_GLOBAL_TEMPLATE: &str = "/get_global_template - get the global template";
static GET_FILTER: &str = "/get_filter url - get the filter for the subscription";
static SET_FILTER: &str = "/set_filter url - set a filter, for example, /set_filter https://www.badykov.com/feed.xml telegram,bots. You'll start receiving posts only containing words in the filter. Use `!word` to stop receiving messages containing the specified `word`. You can combine regular filter words with ! filter words. For example, `!bot,telegram`";
static REMOVE_FILTER: &str = "/remove_filter url - remove the filter for the subscription";
static SET_GLOBAL_FILTER: &str = "/set_global_filter filter - set the global filter";
static GET_GLOBAL_FILTER: &str = "/get_global_filter - get a global filter";
static REMOVE_GLOBAL_FILTER: &str = "/remove_global_filter - remove the global filter";
static UNKNOWN_COMMAND: &str = "unknown command";

static COMMAND: &str = "/help_command";

#[derive(TypedBuilder)]
pub struct HelpCommandInfo {
    args: String,
    message: Message,
}

impl HelpCommandInfo {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    fn command_info(&self) -> SendMessageParams {
        let command = HelpCommand::from_str(&self.args).unwrap();
        let help_for_command = self.help_for_command(command);

        let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        let mut row: Vec<InlineKeyboardButton> = Vec::new();

        let button = InlineKeyboardButton::builder()
            .text("back".to_string())
            .callback_data(Help::command().to_string())
            .build();

        row.push(button);
        buttons.push(row);

        let keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(buttons)
            .build();

        SendMessageParams::builder()
            .chat_id(self.message.chat.id)
            .text(help_for_command)
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            .build()
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    fn help_for_command(&self, command: HelpCommand) -> String {
        match command {
            HelpCommand::Start => START.to_string(),
            HelpCommand::Help => HELP.to_string(),
            HelpCommand::Subscribe => SUBSCRIBE.to_string(),
            HelpCommand::Unsubscribe => UNSUBSCRIBE.to_string(),
            HelpCommand::ListSubscriptions => {
                HelpCommandInfo::fetch_subcriptions(self.message.clone())
            }
            HelpCommand::SetTimezone => SET_TIMEZONE.to_string(),
            HelpCommand::GetTimezone => GET_TIMEZONE.to_string(),
            HelpCommand::SetFilter => SET_FILTER.to_string(),
            HelpCommand::GetFilter => GET_FILTER.to_string(),
            HelpCommand::RemoveFilter => REMOVE_FILTER.to_string(),
            HelpCommand::SetTemplate => SET_TEMPLATE.to_string(),
            HelpCommand::GetTemplate => GET_TEMPLATE.to_string(),
            HelpCommand::RemoveTemplate => REMOVE_TEMPLATE.to_string(),
            HelpCommand::GetGlobalFilter => GET_GLOBAL_FILTER.to_string(),
            HelpCommand::SetGlobalFilter => SET_GLOBAL_FILTER.to_string(),
            HelpCommand::RemoveGlobalFilter => REMOVE_GLOBAL_FILTER.to_string(),
            HelpCommand::GetGlobalTemplate => GET_GLOBAL_TEMPLATE.to_string(),
            HelpCommand::SetGlobalTemplate => SET_GLOBAL_TEMPLATE.to_string(),
            HelpCommand::RemoveGlobalTemplate => REMOVE_GLOBAL_TEMPLATE.to_string(),
            HelpCommand::UnknownCommand => UNKNOWN_COMMAND.to_string(),
        }
    }

    fn fetch_subcriptions(message: Message) -> String {
        let data = match fetch_db_connection(crate::db::pool().clone()) {
            Ok(mut connection) => <crate::bot::commands::list_subscriptions::ListSubscriptions as Command>::list_subscriptions(&mut connection, &message),
            Err(_error_message) => "error fetching data".to_string(),
        };

        data
    }
}

impl Command for HelpCommandInfo {
    fn response(&self) -> Response {
        let params = self.command_info();

        Response::Params(params)
    }

    fn send_message(&self, send_message_params: SendMessageParams) {
        if let Err(error) = self.api().send_message_with_params(&send_message_params) {
            error!(
                "Failed to send a message {:?} {:?}",
                error, send_message_params
            );
        }

        self.remove_message(&self.message);
    }
}

fn fetch_db_connection(
    db_pool: Pool<ConnectionManager<PgConnection>>,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, String> {
    match db_pool.get() {
        Ok(connection) => Ok(connection),
        Err(err) => {
            error!("Failed to fetch a connection from the pool {:?}", err);

            Err("Failed to process your command. Please contact @Ayrat555".to_string())
        }
    }
}
