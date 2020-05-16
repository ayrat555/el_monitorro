use chrono::{DateTime, Utc};
use diesel::connection::Connection;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod feed_items;
pub mod feeds;
pub mod telegram;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn current_time() -> DateTime<Utc> {
    Utc::now()
}
