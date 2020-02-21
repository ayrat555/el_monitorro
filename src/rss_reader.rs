use rss::Channel;

pub struct RssReader {
    pub url: String,
}

impl RssReader {
    pub fn read(&self) -> Channel {
        Channel::from_url(&self.url).unwrap()
    }
}
