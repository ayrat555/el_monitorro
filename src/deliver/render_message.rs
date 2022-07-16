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

const UNICODE_EMPTY_CHARS: [char; 5] = ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'];
const HTML_SPACE: &str = "&#32;";

const BOT_FEED_NAME: &str = "bot_feed_name";
const BOT_ITEM_NAME: &str = "bot_item_name";
const BOT_DATE: &str = "bot_date";
const BOT_FEED_LINK: &str = "bot_feed_link";
const BOT_ITEM_LINK: &str = "bot_item_link";
const BOT_ITEM_DESCRIPTION: &str = "bot_item_description";

const SUBSTRING_HELPER: &str = "substring";
const BOLD_HELPER: &str = "bold";
const ITALIC_HELPER: &str = "italic";

const DEFAULT_TEMPLATE: &str = "{{bot_feed_name}}<br><br>{{bot_item_name}}<br><br>{{bot_item_description}}<br><br>{{bot_date}}<br><br>{{bot_item_link}}<br><br>";
const MAX_CHARS: usize = 4000;

const RENDER_ERROR: &str = "Failed to render template";
const EMPTY_MESSAGE_ERROR: &str = "According to your template the message is empty. Telegram doesn't support empty messages. That's why we're sending this placeholder message.";

handlebars_helper!(substring: |string: String, length: usize| truncate(&string, length));
handlebars_helper!(bold: |string: String| format!("<b>{}</b>", string));
handlebars_helper!(italic: |string: String| format!("<i>{}</i>", string));

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
            .map(|template| self.clean_template(template))
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

        let template_without_html = remove_html(template);
        match reg.render_template(&template_without_html, &data) {
            Err(error) => {
                log::error!("Failed to render template {:?}", error);
                Err(RENDER_ERROR.to_string())
            }
            Ok(result) => Ok(truncate_and_check(&result)),
        }
    }

    fn clean_template(&self, template: String) -> String {
        template.replace("\n", "<br>")
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
            let without_html = remove_html(value.clone());

            return Some(without_html);
        }

        None
    }

    fn maybe_set_value(
        &self,
        map: &mut Map<String, JsonValue>,
        key: &str,
        value_option: &Option<String>,
    ) {
        if let Some(value) = value_option {
            map.insert(key.to_string(), to_json(value));
        }
    }
}

pub fn render_template_example(template: &str) -> Result<String, String> {
    let message_renderer = MessageRenderer::builder()
        .bot_feed_name(Some("feed_name".to_string()))
        .bot_item_name(Some("item_name".to_string()))
        .bot_date(Some(Utc::now().round_subsecs(0)))
        .bot_feed_link(Some("feed_link".to_string()))
        .bot_item_link(Some("item_link".to_string()))
        .bot_item_description(Some("item_description".to_string()))
        .template(Some(template.to_string()))
        .build();

    message_renderer.render()
}

fn truncate_and_check(s: &str) -> String {
    let escaped_data = match decode_html(s) {
        Ok(escaped_html) => escaped_html,
        Err(_) => return RENDER_ERROR.to_string(),
    };

    let truncated_result = truncate(&escaped_data, MAX_CHARS);
    let message_without_empty_chars = remove_empty_characters(&truncated_result);

    if message_without_empty_chars.is_empty() {
        EMPTY_MESSAGE_ERROR.to_string()
    } else {
        message_without_empty_chars
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

fn remove_empty_characters(string: &str) -> String {
    let mut result = string.to_string();
    for character in UNICODE_EMPTY_CHARS {
        result = result.replace(character, "");
    }

    result.replace(HTML_SPACE, "")
}

fn remove_html(string_with_maybe_html: String) -> String {
    nanohtml2text::html2text(&string_with_maybe_html)
}
