use rake::*;

#[derive(Debug)]
pub struct KeywordTagger {
    pub text: String,
    pub stop_words: Option<StopWords>
}

impl KeywordTagger {
    pub fn process(&self) -> Vec<KeywordScore> {
        let stop_words = match &self.stop_words {
            None => StopWords::from_file("./support/english_stopwords").unwrap(),
            Some(stop_words) => stop_words.clone()
        };

        let rake = Rake::new(stop_words);

        rake.run(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::KeywordTagger;
    use rake::KeywordScore;
    use rake::StopWords;

    use std::fs;

    #[test]
    fn it_finds_keywords_in_english_text() {
        let text = fs::read_to_string("./tests/support/text_example").unwrap();
        let keyword_tagger = KeywordTagger { text, stop_words: None };
        let mut result = keyword_tagger.process();

        result = result
            .into_iter()
            .filter(|keyword| keyword.score > 8.0)
            .collect();

        let expected_result = [
            KeywordScore { keyword: "Ethereum based chains Life Balance".to_string(), score: 20.5 },
            KeywordScore { keyword: "create feature requests".to_string(), score: 9.0 },
            KeywordScore { keyword: "active user base".to_string(), score: 8.5 }
        ].to_vec();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn it_finds_keywords_in_russian_text() {
        let stop_words = StopWords::from_file("./support/russian_stopwords").unwrap();
        let text = fs::read_to_string("./tests/support/text_example_russian").unwrap();
        let keyword_tagger = KeywordTagger { text, stop_words: Some(stop_words) };

        let mut result = keyword_tagger.process();
        result = result
            .into_iter()
            .filter(|keyword| keyword.score > 22.0)
            .collect();

        let expected_result = [
            KeywordScore { keyword: "аккаунтах иранских официальных лиц появились изображения флагов Ирана".to_string(), score: 59.5 },
            KeywordScore { keyword: "пишет The New York Times".to_string(), score: 25.0 },
            KeywordScore { keyword: "официально подтверждена гибель Касема Сулеймани".to_string(), score: 22.5 }
        ].to_vec();

        assert_eq!(result, expected_result);
    }
}
