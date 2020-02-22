#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate mockall;
extern crate rake;
extern crate rss;

use rocket_contrib::json::Json;

mod db;
mod keyword_tagger;
mod rss_reader;
mod schema;

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
