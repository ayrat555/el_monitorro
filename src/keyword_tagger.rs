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
