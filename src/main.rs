#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

extern crate rake;

mod keyword_tagger;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/keywords/<text>")]
fn keywords(text: String) -> String {
    text
}

fn main() {
    rocket::ignite().mount("/", routes![index, keywords]).launch();
}
