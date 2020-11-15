use chrono::prelude::*;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenv::dotenv;
use once_cell::sync::OnceCell;
use std::env;

pub mod feed_items;
pub mod feeds;
pub mod telegram;

static POOL: OnceCell<Pool<ConnectionManager<PgConnection>>> = OnceCell::new();

fn pool() -> &'static Pool<ConnectionManager<PgConnection>> {
    POOL.get_or_init(|| {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let database_pool_size_str = env::var("DATABASE_POOL_SIZE").unwrap_or("10".to_string());
        let database_pool_size: u32 = database_pool_size_str.parse().unwrap();

        let manager = ConnectionManager::<PgConnection>::new(database_url);

        Pool::builder()
            .max_size(database_pool_size)
            .build(manager)
            .unwrap()
    })
}

pub fn establish_connection() -> PooledConnection<ConnectionManager<PgConnection>> {
    pool().get().unwrap()
}

pub fn current_time() -> DateTime<Utc> {
    Utc::now().round_subsecs(0)
}
