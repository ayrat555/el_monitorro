use super::Command;
use super::GetFilter;
use super::GetGlobalFilter;
use super::GetGlobalTemplate;
use super::GetTemplate;
use super::GetTimezone;
use super::HelpCommandInfo;
use super::ListSubscriptions;
use super::RemoveFilter;
use super::RemoveGlobalFilter;
use super::RemoveGlobalTemplate;
use super::RemoveTemplate;
use super::Response;
use super::SetFilter;
use super::SetGlobalFilter;
use super::SetGlobalTemplate;
use super::SetTemplate;
use super::SetTimezone;
use super::Start;
use super::Subscribe;
use super::Unsubscribe;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::Message;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use std::fmt;
use std::str::FromStr;
use typed_builder::TypedBuilder;

static COMMAND: &str = "/help";

#[derive(TypedBuilder)]
pub struct Help {
    message: Message,
}

pub enum HelpCommand {
    Help,
    Subscribe,
    Unsubscribe,
    ListSubscriptions,
    Start,
    SetTimezone,
    GetTimezone,
    SetFilter,
    GetFilter,
    RemoveFilter,
    SetTemplate,
    GetTemplate,
    RemoveTemplate,
    GetGlobalFilter,
    SetGlobalFilter,
    RemoveGlobalFilter,
    GetGlobalTemplate,
    SetGlobalTemplate,
    RemoveGlobalTemplate,
    UnknownCommand,
}

impl fmt::Display for HelpCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &HelpCommand::Start => write!(f, "{}", Start::command()),
            &HelpCommand::Help => write!(f, "{}", Help::command()),
            &HelpCommand::Subscribe => write!(f, "{}", Subscribe::command()),
            &HelpCommand::Unsubscribe => write!(f, "{}", Unsubscribe::command()),
            &HelpCommand::ListSubscriptions => write!(f, "{}", ListSubscriptions::command()),
            &HelpCommand::SetTimezone => write!(f, "{}", SetTimezone::command()),
            &HelpCommand::GetTimezone => write!(f, "{}", GetTimezone::command()),
            &HelpCommand::SetFilter => write!(f, "{}", SetFilter::command()),
            &HelpCommand::GetFilter => write!(f, "{}", GetFilter::command()),
            &HelpCommand::RemoveFilter => write!(f, "{}", RemoveFilter::command()),
            &HelpCommand::SetTemplate => write!(f, "{}", SetTemplate::command()),
            &HelpCommand::GetTemplate => write!(f, "{}", GetTemplate::command()),
            &HelpCommand::RemoveTemplate => write!(f, "{}", RemoveTemplate::command()),
            &HelpCommand::GetGlobalFilter => write!(f, "{}", GetGlobalFilter::command()),
            &HelpCommand::SetGlobalFilter => write!(f, "{}", SetGlobalFilter::command()),
            &HelpCommand::RemoveGlobalFilter => write!(f, "{}", RemoveGlobalFilter::command()),
            &HelpCommand::GetGlobalTemplate => write!(f, "{}", GetGlobalTemplate::command()),
            &HelpCommand::SetGlobalTemplate => write!(f, "{}", SetGlobalTemplate::command()),
            &HelpCommand::RemoveGlobalTemplate => write!(f, "{}", RemoveGlobalTemplate::command()),
            // just a placeholder
            &HelpCommand::UnknownCommand => write!(f, "{}", "/unknown_command"),
        }
    }
}

impl FromStr for HelpCommand {
    type Err = ();

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let command = if command.starts_with(Help::command()) {
            HelpCommand::Help
        } else if command.starts_with(Subscribe::command()) {
            HelpCommand::Subscribe
        } else if command.starts_with(Unsubscribe::command()) {
            HelpCommand::Unsubscribe
        } else if command.starts_with(ListSubscriptions::command()) {
            HelpCommand::ListSubscriptions
        } else if command.starts_with(Start::command()) {
            HelpCommand::Start
        } else if command.starts_with(SetTimezone::command()) {
            HelpCommand::SetTimezone
        } else if command.starts_with(GetTimezone::command()) {
            HelpCommand::GetTimezone
        } else if command.starts_with(SetFilter::command()) {
            HelpCommand::SetFilter
        } else if command.starts_with(GetFilter::command()) {
            HelpCommand::GetFilter
        } else if command.starts_with(RemoveFilter::command()) {
            HelpCommand::RemoveFilter
        } else if command.starts_with(SetTemplate::command()) {
            HelpCommand::SetTemplate
        } else if command.starts_with(GetTemplate::command()) {
            HelpCommand::GetTemplate
        } else if command.starts_with(RemoveTemplate::command()) {
            HelpCommand::RemoveTemplate
        } else if command.starts_with(SetGlobalFilter::command()) {
            HelpCommand::SetGlobalFilter
        } else if command.starts_with(RemoveGlobalTemplate::command()) {
            HelpCommand::RemoveGlobalTemplate
        } else if command.starts_with(GetGlobalTemplate::command()) {
            HelpCommand::GetGlobalTemplate
        } else if command.starts_with(SetGlobalTemplate::command()) {
            HelpCommand::SetGlobalTemplate
        } else if command.starts_with(GetGlobalFilter::command()) {
            HelpCommand::GetGlobalFilter
        } else if command.starts_with(RemoveGlobalFilter::command()) {
            HelpCommand::RemoveGlobalFilter
        } else {
            HelpCommand::UnknownCommand
        };

        Ok(command)
    }
}

impl Help {
    pub fn run(&self) {
        self.execute(&self.message);
    }

    pub fn command() -> &'static str {
        COMMAND
    }

    pub fn help_keyboard_params(&self) -> SendMessageParams {
        let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for command in [
            HelpCommand::Help,
            HelpCommand::Subscribe,
            HelpCommand::Unsubscribe,
            HelpCommand::ListSubscriptions,
            HelpCommand::Start,
            HelpCommand::SetTimezone,
            HelpCommand::GetTimezone,
            HelpCommand::SetFilter,
            HelpCommand::GetFilter,
            HelpCommand::RemoveFilter,
            HelpCommand::SetTemplate,
            HelpCommand::GetTemplate,
            HelpCommand::RemoveTemplate,
            HelpCommand::GetGlobalFilter,
            HelpCommand::SetGlobalFilter,
            HelpCommand::RemoveGlobalFilter,
            HelpCommand::GetGlobalTemplate,
            HelpCommand::SetGlobalTemplate,
            HelpCommand::RemoveGlobalTemplate,
        ] {
            let mut row: Vec<InlineKeyboardButton> = Vec::new();

            let button = InlineKeyboardButton::builder()
                .text(command.to_string())
                .callback_data(format!("{} {}", HelpCommandInfo::command(), command))
                .build();

            row.push(button);
            buttons.push(row);
        }

        let keyboard = InlineKeyboardMarkup::builder()
            .inline_keyboard(buttons)
            .build();

        SendMessageParams::builder()
            .chat_id(self.message.chat.id)
            .text("Select a command:")
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            .build()
    }
}

impl Command for Help {
    fn response(&self) -> Response {
        let params = self.help_keyboard_params();

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
