pub struct ShowFeedKeyboard {
    chat_id: i64,
    feed_url: String,
}

impl Command for ShowFeedKeyboard {}

impl ShowFeedKeyboard {
    pub fn run(&self) -> String {
        let subscription = match self.find_subscription(db_connection, self.chat_id, self.feed_url)
        {
            Err(message) => return message,
            Ok(subscription) => subscription,
        };
    }
}
