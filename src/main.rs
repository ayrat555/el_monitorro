use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot;
use el_monitorro::sync::sync_job;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    tokio::spawn(sync_job::sync_all_feeds());

    match bot::api::start_bot().await {
        Err(_) => log::error!("Couldn't start a bot"),
        _ => (),
    };
}
