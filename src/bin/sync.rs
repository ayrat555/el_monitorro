use dotenv::dotenv;
use el_monitorro;
use el_monitorro::sync::sync_job;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    sync_job::sync_feeds().await;
}
