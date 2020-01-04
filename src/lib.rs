#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

extern crate rake;

pub mod keyword_tagger;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
