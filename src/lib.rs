#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rake;

use rocket_contrib::json::Json;

pub mod keyword_tagger;

use keyword_tagger::*;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/keywords", data = "<text>")]
fn keywords(text: String) -> Json<Vec<Keyword>> {
    let keyword_tagger = KeywordTagger {
        text,
        stop_words: None,
    };
    let keywords = keyword_tagger.process();

    Json(keywords)
}

pub fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/api", routes![index, keywords])
}
