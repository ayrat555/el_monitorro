use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot::deliver_job;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    deliver_job::deliver_updates().await;
}
