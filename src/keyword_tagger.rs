use rake::*;

use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug)]
pub struct KeywordTagger {
    pub text: String,
    pub stop_words: Option<StopWords>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyword {
    pub value: String,
    pub score: f64,
}

impl From<KeywordScore> for Keyword {
    fn from(keyword_score: KeywordScore) -> Self {
        Keyword {
            score: keyword_score.score,
            value: keyword_score.keyword,
        }
    }
}

impl KeywordTagger {
    pub fn process(&self) -> Vec<Keyword> {
        let stop_words = match &self.stop_words {
            None => StopWords::from_file("./support/english_stopwords").unwrap(),
            Some(stop_words) => stop_words.clone(),
        };

        let rake = Rake::new(stop_words);

        rake.run(&self.text)
            .into_iter()
            .map(|i| Keyword::from(i))
            .collect::<Vec<Keyword>>()
    }
}

impl Serialize for Keyword {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Keyword", 2)?;
        s.serialize_field("keyword", &self.value)?;
        s.serialize_field("score", &self.score)?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{Keyword, KeywordTagger};
    use rake::StopWords;

    use std::fs;

    #[test]
    fn it_finds_keywords_in_english_text() {
        let text = fs::read_to_string("./tests/support/text_example").unwrap();
        let keyword_tagger = KeywordTagger {
            text,
            stop_words: None,
        };
        let mut result = keyword_tagger.process();

        result = result
            .into_iter()
            .filter(|keyword| keyword.score > 8.0)
            .collect();

        let expected_result = [
            Keyword {
                value: "Ethereum based chains Life Balance".to_string(),
                score: 20.5,
            },
            Keyword {
                value: "create feature requests".to_string(),
                score: 9.0,
            },
            Keyword {
                value: "active user base".to_string(),
                score: 8.5,
            },
        ]
        .to_vec();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn it_finds_keywords_in_russian_text() {
        let stop_words = StopWords::from_file("./support/russian_stopwords").unwrap();
        let text = fs::read_to_string("./tests/support/text_example_russian").unwrap();
        let keyword_tagger = KeywordTagger {
            text,
            stop_words: Some(stop_words),
        };

        let mut result = keyword_tagger.process();
        result = result
            .into_iter()
            .filter(|keyword| keyword.score > 22.0)
            .collect();

        let expected_result = [
            Keyword {
                value: "аккаунтах иранских официальных лиц появились изображения флагов Ирана"
                    .to_string(),
                score: 59.5,
            },
            Keyword {
                value: "пишет The New York Times".to_string(),
                score: 25.0,
            },
            Keyword {
                value: "официально подтверждена гибель Касема Сулеймани".to_string(),
                score: 22.5,
            },
        ]
        .to_vec();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn it_serializes_keyword() {
        let keyword = Keyword {
            value: "value".to_string(),
            score: 5.0,
        };

        let string_keyword = serde_json::to_string(&keyword).unwrap();

        let expected_result = "{\"keyword\":\"value\",\"score\":5.0}";
        assert_eq!(string_keyword, expected_result);
    }
}
