#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket_contrib::json::Json;

extern crate rake;

mod keyword_tagger;

use keyword_tagger::*;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/keywords", data = "<text>")]
fn keywords(text: String) -> Json<Vec<Keyword>> {
    let keyword_tagger = KeywordTagger { text, stop_words: None };
    let keywords = keyword_tagger.process();

    Json(keywords)
}

fn main() {
    rocket::ignite().mount("/", routes![index, keywords]).launch();
}
