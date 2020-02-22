use mockall::*;
use rss::Channel;
use rss::Error;

pub struct RssReader {
    pub url: String,
}

#[automock]
pub trait ReadRSS {
    fn read_rss(&self) -> Result<Channel, Error>;
}

impl ReadRSS for RssReader {
    fn read_rss(&self) -> Result<Channel, Error> {
        Channel::from_url(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::MockReadRSS;
    use super::ReadRSS;
    use rss::Channel;
    use std::fs;
    use std::str::FromStr;

    #[test]
    fn it_mocks_read_rss_trait() {
        let mut mock = MockReadRSS::new();
        let xml_feed = fs::read_to_string("./tests/support/rss_feed_example.xml").unwrap();

        mock.expect_read_rss()
            .returning(move || Channel::from_str(&xml_feed));

        assert!(mock.read_rss().is_ok());
    }
}
