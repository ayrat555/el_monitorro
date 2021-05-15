use dotenv::dotenv;

use el_monitorro::bot;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    bot::api::start_bot().await;
}
