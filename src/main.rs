use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    match bot::api::start_bot().await {
        Err(_) => log::error!("Couldn't start a bot"),
        _ => (),
    };
}
