use aho_corasick::AhoCorasickBuilder;
use aho_corasick::MatchKind;
use chrono::offset::FixedOffset;
use chrono::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use handlebars::handlebars_helper;
use handlebars::to_json;
use handlebars::Handlebars;
use handlebars::JsonValue;
use htmlescape::decode_html;
use serde_json::value::Map;
use typed_builder::TypedBuilder as Builder;

const BOT_FEED_NAME: &str = "bot_feed_name";
const BOT_ITEM_NAME: &str = "bot_item_name";
const BOT_DATE: &str = "bot_date";
const BOT_FEED_LINK: &str = "bot_feed_link";
const BOT_ITEM_LINK: &str = "bot_item_link";
const BOT_ITEM_DESCRIPTION: &str = "bot_item_description";

const SUBSTRING_HELPER: &str = "substring";
const CREATE_LINK_HELPER: &str = "create_link";
const BOLD_HELPER: &str = "bold";
const ITALIC_HELPER: &str = "italic";

const DEFAULT_TEMPLATE: &str = "{{bot_feed_name}}\n\n{{bot_item_name}}\n\n{{bot_item_description}}\n\n{{bot_date}}\n\n{{bot_item_link}}\n\n";
const MAX_MESSAGE_CHARS: usize = 4000;
const MAX_ITEM_CHARS: usize = 3000;
const MAX_LINK_CHARS: usize = 1000;

const RENDER_ERROR: &str = "Failed to render template";
const EMPTY_MESSAGE_ERROR: &str = "According to your template the message is empty. Telegram doesn't support empty messages. That's why we're sending this placeholder message.";

handlebars_helper!(create_link: |string: String, link: String| render_link(&string,&link));
handlebars_helper!(bold: |string: String| format!("<b>{}</b>", string));
handlebars_helper!(italic: |string: String| format!("<i>{}</i>", string));
handlebars_helper!(substring: |string: String, length: usize| truncate(&string, length));

#[derive(Builder)]
pub struct MessageRenderer {
    #[builder(setter(into), default)]
    bot_feed_name: Option<String>,
    #[builder(setter(into), default)]
    bot_item_name: Option<String>,
    #[builder(setter(into), default)]
    bot_date: Option<DateTime<Utc>>,
    #[builder(setter(into), default)]
    bot_feed_link: Option<String>,
    #[builder(setter(into), default)]
    bot_item_link: Option<String>,
    #[builder(setter(into), default)]
    bot_item_description: Option<String>,
    #[builder(setter(into), default)]
    template: Option<String>,
    #[builder(setter(into), default)]
    offset: Option<i32>,
}

impl MessageRenderer {
    pub fn render(&self) -> Result<String, String> {
        let template = self
            .template
            .clone()
            .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());

        let mut data = Map::new();

        self.maybe_set_value(
            &mut data,
            BOT_FEED_NAME,
            &self.maybe_remove_html(&self.bot_feed_name),
        );
        self.maybe_set_value(
            &mut data,
            BOT_ITEM_NAME,
            &self.maybe_remove_html(&self.bot_item_name),
        );
        self.maybe_set_value(&mut data, BOT_DATE, &self.date());
        self.maybe_set_value(&mut data, BOT_FEED_LINK, &self.bot_feed_link);
        self.maybe_set_value(&mut data, BOT_ITEM_LINK, &self.bot_item_link);
        self.maybe_set_value(
            &mut data,
            BOT_ITEM_DESCRIPTION,
            &self.maybe_remove_html(&self.bot_item_description),
        );

        let mut reg = Handlebars::new();

        reg.register_helper(SUBSTRING_HELPER, Box::new(substring));
        reg.register_helper(BOLD_HELPER, Box::new(bold));
        reg.register_helper(ITALIC_HELPER, Box::new(italic));
        reg.register_helper(CREATE_LINK_HELPER, Box::new(create_link));

        match reg.render_template(&template, &data) {
            Err(error) => {
                log::error!("Failed to render template {:?}", error);
                Err(RENDER_ERROR.to_string())
            }
            Ok(result) => Ok(truncate_and_check(&result)),
        }
    }

    fn date(&self) -> Option<String> {
        if let Some(date) = &self.bot_date {
            let time_offset = match self.offset {
                None => FixedOffset::west(0),
                Some(value) => {
                    if value > 0 {
                        FixedOffset::east(value * 60)
                    } else {
                        FixedOffset::west(-value * 60)
                    }
                }
            };

            let date_with_timezone = date.with_timezone(&time_offset);

            return Some(format!("{}", date_with_timezone));
        }

        None
    }

    fn maybe_remove_html(&self, value_option: &Option<String>) -> Option<String> {
        if let Some(value) = value_option {
            let without_html = remove_html(value);
            let truncated = truncate(&without_html, MAX_ITEM_CHARS);

            return Some(truncated);
        }

        None
    }

    fn maybe_set_value(
        &self,
        map: &mut Map<String, JsonValue>,
        key: &str,
        value_option: &Option<String>,
    ) {
        match value_option {
            Some(value) => map.insert(key.to_string(), to_json(value)),
            None => map.insert(key.to_string(), to_json("".to_string())),
        };
    }
}

pub fn render_template_example(template: &str) -> Result<String, String> {
    let message_renderer = MessageRenderer::builder()
        .bot_feed_name(Some("feed_name".to_string()))
        .bot_item_name(Some("item_name".to_string()))
        .bot_date(Some(Utc::now().round_subsecs(0)))
        .bot_feed_link(Some("https://www.badykov.com/feed.xml".to_string()))
        .bot_item_link(Some("https://www.badykov.com/".to_string()))
        .bot_item_description(Some("item_description".to_string()))
        .template(Some(template.to_string()))
        .build();

    message_renderer.render()
}

fn render_link(s: &str, l: &str) -> String {
    let value = if s.is_empty() {
        "link".to_string()
    } else {
        truncate(s, MAX_LINK_CHARS)
    };
    format!("<a href=\"{}\">{}</a>", l, value)
}

fn truncate_and_check(s: &str) -> String {
    let escaped_data = match decode_html(s) {
        Ok(escaped_html) => escaped_html,
        Err(_) => return RENDER_ERROR.to_string(),
    };

    let truncated_result = truncate(&escaped_data, MAX_MESSAGE_CHARS);

    if truncated_result.is_empty() {
        EMPTY_MESSAGE_ERROR.to_string()
    } else {
        truncated_result
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    let result = match s.char_indices().nth(max_chars) {
        None => String::from(s),
        Some((idx, _)) => {
            let mut string = String::from(&s[..idx]);

            string.push_str("...");

            string
        }
    };

    result.trim().to_string()
}

fn remove_html(string_with_maybe_html: &str) -> String {
    let string_without_html = nanohtml2text::html2text(string_with_maybe_html);

    let ac = AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostFirst)
        .build(&[
            "&#32;", "&", "<", ">", "\u{200B}", "\u{200C}", "\u{200D}", "\u{2060}", "\u{FEFF}",
        ]);

    ac.replace_all(
        &string_without_html,
        &[" ", "&amp;", "&lt;", "&gt;", " ", " ", " ", " ", " "],
    )
}
