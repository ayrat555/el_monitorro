use dotenv::dotenv;
use el_monitorro;
use el_monitorro::cleaner::clean_job;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    clean_job::clean().await;
}
