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
use super::SetContentFields;
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

static HELP: &str =
        "/start - show the description of the bot and its contact information\n\n\
         /subscribe url - subscribe to the feed\n\n\
         /unsubscribe url - unsubscribe from the feed\n\n\
         /list_subscriptions - list your subscriptions\n\n\
         /help - show available commands\n\n\
         /set_timezone - set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, your offset is equal to 60 x 10 = 600\n\n\
         /get_timezone - get your timezone\n\n\
         /set_template url template - set a template for all received feed items for the specified subscription. All new updates will be converted to the format defined by this subscription. Supported fields you can use for templates:\n\
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
         - `bold` helper. Usage:  {{bold bot_item_name}}\n\n\
         /get_template url - get the template for the subscription\n\n\
         /remove_template url - remove the template\n\n\
         /set_global_template template - set global template. This template will be used for all subscriptions. If the subscription has its own template, it will be used instead. See /set_template for available fields.\n\n\
         /remove_global_template - remove global template\n\n\
         /get_global_template - get global template\n\n\
         /get_filter url - get the filter for the subscription\n\n\
         /set_filter url template - set filter, for example, /set_filter https://www.badykov.com/feed.xml telegram,bots. You'll start receiving posts only containing words in the filter. Use `!word` to stop receiving messages containing the specified `word`. You can combine regular filter words with ! filter words. For example, `!bot,telegram`\n\n\
         /remove_filter url - remove filter\n\n\
         /set_global_filter filter - set global filter\n\n\
         /get_global_filter - get a global filter\n\n\
         /remove_global_filter - remove global filter\n\n";

static START: &str = "/start - show the description of the bot and its contact information";
static SUBSCRIBE: &str = "/subscribe url - subscribe to a feed";
static UNSUBSCRIBE: &str = "/unsubscribe url - unsubscribe from a feed";
static LIST_SUBSCRIPTIONS: &str = "/list_subscriptions - list your subscriptions";
static SET_TIMEZONE: &str = "set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, your offset is equal to 60 x 10 = 600";
static GET_TIMEZONE: &str = "/get_timezone - get your timezone";

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
    SetContentFields,
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
            &HelpCommand::SetContentFields => write!(f, "{}", SetContentFields::command()),
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
        } else if command.starts_with(SetContentFields::command()) {
            HelpCommand::SetContentFields
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
            HelpCommand::SetContentFields,
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
}
