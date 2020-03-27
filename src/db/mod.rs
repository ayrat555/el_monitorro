use chrono::prelude::{DateTime, Utc};
use diesel::connection::Connection;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use rocket_contrib::databases::diesel::PgConnection as RocketPgConnection;
use std::env;

pub mod feed_items;
pub mod feeds;
pub mod telegram;

#[database("diesel_postgres_pool")]
pub struct Conn(RocketPgConnection);

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn current_time() -> DateTime<Utc> {
    Utc::now()
}
