use rake::*;

pub struct KeywordTagger {
    pub text: String
}

impl KeywordTagger {
    pub fn process(&self) -> Vec<KeywordScore> {
        let stop_words = StopWords::from_file("./support/english_stopwords").unwrap();
        let rake = Rake::new(stop_words);

        rake.run(&self.text)
    }
}
