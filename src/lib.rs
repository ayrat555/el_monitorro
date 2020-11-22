#[macro_use]
extern crate diesel;

#[macro_use]
extern crate failure;

pub mod bot;
pub mod cleaner;
pub mod db;
mod models;
mod schema;
pub mod sync;
