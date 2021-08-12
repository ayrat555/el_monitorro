use super::Command;
use super::Message;
use crate::bot::telegram_client::Api;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

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
         - bot_space - defines a space character\n\
         - bot_new_line - defines a new line character\n\
         Example: /set_template https://www.badykov.com/feed.xml bot_datebot_spacebot_item_namebot_new_linebot_item_description\n\n\
         /get_template url - get the template for the subscription\n\n\
         /set_global_template template - set global template. This template will be used for all subscriptions. If the subscription has its own template, it will be used instead. See /set_template for available fields.\n\n\
         /get_global_template - get global template\n\n\
         /get_filter url - get the filter for the subscription\n\n\
         /set_filter url template - set filter, for example, /set_filter https://www.badykov.com/feed.xml telegram,bots. You'll start receiving posts only containing words in the filter. Use `!word` to stop receiving messages containing the specified `word`. You can combine regular filter words with ! filter words. For example, `!bot,telegram`\n\n\
         /remove_filter url - remove filter\n\n";

static COMMAND: &str = "/help";

pub struct Help {}

impl Help {
    pub fn execute(db_pool: Pool<ConnectionManager<PgConnection>>, api: Api, message: Message) {
        Self {}.execute(db_pool, api, message);
    }

    pub fn command() -> &'static str {
        COMMAND
    }
}

impl Command for Help {
    fn response(
        &self,
        _db_pool: Pool<ConnectionManager<PgConnection>>,
        _message: &Message,
    ) -> String {
        HELP.to_string()
    }

    fn command(&self) -> &str {
        Self::command()
    }
}
