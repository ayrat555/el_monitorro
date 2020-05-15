#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
extern crate log;
#[macro_use]
extern crate failure;
extern crate dotenv;
extern crate futures;
extern crate mockall;
extern crate rss;
extern crate telegram_bot;
extern crate tokio;
extern crate url;

pub mod bot;
mod db;
mod models;
mod schema;
pub mod sync;
