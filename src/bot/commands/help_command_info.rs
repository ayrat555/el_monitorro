use super::help::HelpCommand;
use super::Command;
use super::Help;
use super::Response;
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
static LIST_SUBSCRIPTIONS: &str = "/list_subscriptions - list your subscriptions";
static SET_TIMEZONE: &str = "set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, your offset is equal to 60 x 10 = 600";
static GET_TIMEZONE: &str = "/get_timezone - get your timezone";
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
            .text("Back".to_string())
            .callback_data(format!("{}", Help::command()))
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
            HelpCommand::Help => START.to_string(),
            HelpCommand::Subscribe => START.to_string(),
            HelpCommand::Unsubscribe => START.to_string(),
            HelpCommand::ListSubscriptions => START.to_string(),
            HelpCommand::SetTimezone => START.to_string(),
            HelpCommand::GetTimezone => START.to_string(),
            HelpCommand::SetFilter => START.to_string(),
            HelpCommand::GetFilter => START.to_string(),
            HelpCommand::RemoveFilter => START.to_string(),
            HelpCommand::SetTemplate => START.to_string(),
            HelpCommand::GetTemplate => START.to_string(),
            HelpCommand::RemoveTemplate => START.to_string(),
            HelpCommand::GetGlobalFilter => START.to_string(),
            HelpCommand::SetGlobalFilter => START.to_string(),
            HelpCommand::RemoveGlobalFilter => START.to_string(),
            HelpCommand::GetGlobalTemplate => START.to_string(),
            HelpCommand::SetGlobalTemplate => START.to_string(),
            HelpCommand::RemoveGlobalTemplate => START.to_string(),
            HelpCommand::SetContentFields => START.to_string(),
            HelpCommand::UnknownCommand => START.to_string(),
        }
    }
}

impl Command for HelpCommandInfo {
    fn response(&self) -> Response {
        let params = self.command_info();

        Response::Params(params)
    }
}
