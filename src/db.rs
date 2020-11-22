use chrono::prelude::*;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenv::dotenv;
use once_cell::sync::OnceCell;
use std::env;
use tokio::sync::Semaphore;
use tokio::sync::SemaphorePermit;

pub mod feed_items;
pub mod feeds;
pub mod telegram;

static POOL: OnceCell<Pool<ConnectionManager<PgConnection>>> = OnceCell::new();
static SEMAPHORE: OnceCell<Semaphore> = OnceCell::new();
static POOL_NUMBER: OnceCell<usize> = OnceCell::new();

pub struct SemaphoredDbConnection<'a> {
    _semaphore_permit: SemaphorePermit<'a>,
    pub connection: PooledConnection<ConnectionManager<PgConnection>>,
}

pub async fn get_semaphored_connection<'a>() -> SemaphoredDbConnection<'a> {
    let _semaphore_permit = semaphore().acquire().await;
    let connection = establish_connection();

    SemaphoredDbConnection {
        _semaphore_permit,
        connection,
    }
}

pub fn current_time() -> DateTime<Utc> {
    Utc::now().round_subsecs(0)
}

pub fn establish_connection() -> PooledConnection<ConnectionManager<PgConnection>> {
    pool().get().unwrap()
}

pub fn semaphore() -> &'static Semaphore {
    SEMAPHORE.get_or_init(|| Semaphore::new(*pool_connection_number()))
}

pub fn pool_connection_number() -> &'static usize {
    POOL_NUMBER.get_or_init(|| {
        dotenv().ok();

        let database_pool_size_str = env::var("DATABASE_POOL_SIZE").unwrap_or("10".to_string());
        let database_pool_size: usize = database_pool_size_str.parse().unwrap();

        database_pool_size
    })
}

fn pool() -> &'static Pool<ConnectionManager<PgConnection>> {
    POOL.get_or_init(|| {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let manager = ConnectionManager::<PgConnection>::new(database_url);

        Pool::builder()
            .max_size(*pool_connection_number() as u32)
            .build(manager)
            .unwrap()
    })
}
