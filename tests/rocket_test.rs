mod common;

use common::*;
use el_monitorro::keyword_tagger::KeywordTagger;
use std::fs;

#[test]
fn test_get_profile() {
    let text = fs::read_to_string("./tests/support/text_small").unwrap();

    let client = test_client();
    let response = &mut client.post("/api/keywords").body(&text).dispatch();

    let response_json_value = response_json_value(response);

    let keyword_tagger = KeywordTagger {
        text,
        stop_words: None,
    };

    let result = keyword_tagger.process();
    assert!(result.len() > 2);

    let expected_json_result = serde_json::to_string(&result).unwrap();

    assert_eq!(format!("{}", response_json_value), expected_json_result);
}
