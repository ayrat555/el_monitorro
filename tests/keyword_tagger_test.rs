#![feature(drain_filter)]

use std::fs;

use el_monitorro::keyword_tagger::KeywordTagger;
use rake::KeywordScore;

#[test]
fn it_finds_keywords_in_english_text() {
    let text = fs::read_to_string("./tests/support/text_example").unwrap();
    let keyword_tagger = KeywordTagger { text };
    let mut result = keyword_tagger.process();

    result.drain_filter(|keyword| keyword.score <= 8.0);

    let expected_result = [
        KeywordScore { keyword: "Ethereum based chains Life Balance".to_string(), score: 20.5 },
        KeywordScore { keyword: "Users find bugs".to_string(), score: 9.0 },
        KeywordScore { keyword: "popular small libraries".to_string(), score: 9.0 },
        KeywordScore { keyword: "create feature requests".to_string(), score: 9.0 },
        KeywordScore { keyword: "active user base".to_string(), score: 8.5 }
    ].to_vec();

    assert_eq!(result, expected_result);
}
