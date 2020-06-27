#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
extern crate log;
#[macro_use]
extern crate failure;
extern crate atom_syndication;
extern crate dotenv;
extern crate feed_rs;
extern crate futures;
extern crate isahc;
extern crate rss;
extern crate serde_json;
extern crate telegram_bot;
extern crate tokio;
extern crate url;

pub mod bot;
mod db;
mod models;
mod schema;
pub mod sync;
