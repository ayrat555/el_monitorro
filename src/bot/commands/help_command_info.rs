use super::help::HelpCommand;
use super::Command;
use super::Help;
use super::Response;
use frankenstein::Message;
use frankenstein::SendMessageParams;
use std::str::FromStr;
use typed_builder::TypedBuilder;

static START: &str = "/start - show the description of the bot and its contact information";
static SUBSCRIBE: &str = "/subscribe url - subscribe to a feed";
static UNSUBSCRIBE: &str = "/unsubscribe url - unsubscribe from a feed";
static LIST_SUBSCRIPTIONS: &str = "/list_subscriptions - list your subscriptions";
static HELP: &str = "/help - show available commands";
static SET_TIMEZONE: &str = "/set_timezone timezone_minutes - set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, your offset is equal to 60 x 10 = 600";
static GET_TIMEZONE: &str = "/get_timezone - get your timezone";
static GET_TEMPLATE: &str = "/get_template feed_url - get the template for the subscription";
static SET_TEMPLATE: &str =
    "/set_template url template - set a template for all received feed items for the specified subscription. All new updates will be converted to the format defined by this subscription. Supported fields you can use for templates:\n\
     - bot_feed_name - name of the feed\n\
     - bot_feed_link - url of the feed\n\
     - bot_item_name - name of the item\n\
     - bot_item_link - url of the item\n\
     - bot_item_description - description of the item\n\
     - bot_item_author - author of the item\n\
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
static GET_PREVIEW_ENABLED: &str = "/get_preview_enabled - check if previews are enabled for the current chat. by default, previews are enabled";
static TOGGLE_PREVIEW_ENABLED: &str = "/toggle_preview_enabled - disable or enable previews";
static UNKNOWN_COMMAND: &str = "unknown command";

static COMMAND: &str = "/help_command";

#[derive(TypedBuilder)]
pub struct HelpCommandInfo {
    args: String,
    message: Message,
}

impl HelpCommandInfo {
    pub fn run(&self) {
        self.execute(
            &self.message,
            &format!("{} - {}", Self::command(), self.args),
        );
    }

    fn command_info(&self) -> String {
        let command = HelpCommand::from_str(&self.args).unwrap();

        self.help_for_command(command)
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
            HelpCommand::ListSubscriptions => LIST_SUBSCRIPTIONS.to_string(),
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
            HelpCommand::GetPreviewEnabled => GET_PREVIEW_ENABLED.to_string(),
            HelpCommand::TogglePreviewEnabled => TOGGLE_PREVIEW_ENABLED.to_string(),
            HelpCommand::UnknownCommand => UNKNOWN_COMMAND.to_string(),
        }
    }
}

impl Command for HelpCommandInfo {
    fn response(&self) -> Response {
        let help_for_command = self.command_info();

        self.simple_keyboard(help_for_command, Help::command().to_string(), &self.message)
    }

    fn send_message(&self, send_message_params: SendMessageParams) {
        self.send_message_and_remove(send_message_params, &self.message);
    }
}
