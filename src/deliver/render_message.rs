use handlebars::handlebars_helper;
use handlebars::to_json;
use handlebars::Handlebars;
use handlebars::JsonValue;
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
const DEFAULT_TEMPLATE: &str = "{{bot_feed_name}}\n\n{{bot_item_name}}\n\n{{bot_item_description}}\n\n{{bot_date}}\n\n{{bot_item_link}}\n\n";
const MAX_CHARS: usize = 4000;

const RENDER_ERROR: &str = "Failed to render template";
const EMPTY_MESSAGE_ERROR: &str = "According to your template the message is empty. Telegram doesn't support empty messages. That's why we're sending this placeholder message.";

handlebars_helper!(substring: |string: String, length: usize| truncate(&string, length));

#[derive(Builder)]
pub struct MessageRenderer {
    #[builder(setter(into, strip_option), default)]
    bot_feed_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    bot_item_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    bot_date: Option<String>,
    #[builder(setter(into, strip_option), default)]
    bot_feed_link: Option<String>,
    #[builder(setter(into, strip_option), default)]
    bot_item_link: Option<String>,
    #[builder(setter(into, strip_option), default)]
    bot_item_description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    template: Option<String>,
}

impl MessageRenderer {
    pub fn render(&self) -> Result<String, String> {
        let template = self
            .template
            .clone()
            .unwrap_or(DEFAULT_TEMPLATE.to_string());

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
        self.maybe_set_value(&mut data, BOT_DATE, &self.bot_date);
        self.maybe_set_value(&mut data, BOT_FEED_LINK, &self.bot_feed_link);
        self.maybe_set_value(&mut data, BOT_ITEM_LINK, &self.bot_item_link);
        self.maybe_set_value(
            &mut data,
            BOT_ITEM_DESCRIPTION,
            &self.maybe_remove_html(&self.bot_item_description),
        );

        let mut reg = Handlebars::new();
        reg.register_helper(SUBSTRING_HELPER, Box::new(substring));

        match reg.render_template(&template, &data) {
            Err(error) => {
                log::error!("Failed to render template {:?}", error);
                Err(RENDER_ERROR.to_string())
            }
            Ok(result) => Ok(truncate_and_check(&result)),
        }
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
        .bot_feed_name("feed_name")
        .bot_item_name("item_name")
        .bot_date("date")
        .bot_feed_link("feed_link")
        .bot_item_link("item_link")
        .bot_item_description("item_description")
        .template(template)
        .build();

    message_renderer.render()
}

fn truncate_and_check(s: &str) -> String {
    let truncated_result = truncate(s, MAX_CHARS);

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

    let trimmed_result = result.trim();

    remove_empty_characters(trimmed_result)
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
