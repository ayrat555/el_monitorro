use crate::config::Config;
use chrono::prelude::*;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::r2d2;
use once_cell::sync::OnceCell;

#[cfg(test)]
use diesel::connection::Connection;

#[cfg(test)]
use dotenv::dotenv;

pub mod feed_items;
pub mod feeds;
pub mod telegram;

static POOL: OnceCell<r2d2::Pool<r2d2::ConnectionManager<PgConnection>>> = OnceCell::new();

#[cfg(test)]
pub fn establish_test_connection() -> PgConnection {
    dotenv().ok();

    let url = database_url();

    PgConnection::establish(&url).unwrap_or_else(|_| panic!("Error connecting to {}", url))
}

pub fn current_time() -> DateTime<Utc> {
    Utc::now().round_subsecs(0)
}

pub fn pool() -> &'static r2d2::Pool<r2d2::ConnectionManager<PgConnection>> {
    POOL.get_or_init(create_connection_pool)
}

pub fn create_connection_pool() -> r2d2::Pool<r2d2::ConnectionManager<PgConnection>> {
    let url = database_url();

    let manager = r2d2::ConnectionManager::<PgConnection>::new(url);

    r2d2::Pool::builder()
        .max_size(Config::commands_db_pool_number())
        .build(manager)
        .unwrap()
}

pub fn database_url() -> String {
    Config::database_url()
}
