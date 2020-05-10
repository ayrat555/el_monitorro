use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot;
use el_monitorro::bot::deliver_job;
use el_monitorro::sync::sync_job;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    tokio::spawn(deliver_job::deliver_updates_every_hour());
    tokio::spawn(sync_job::sync_feeds_every_hour());

    match bot::api::start_bot().await {
        Err(_) => log::error!("Couldn't start a bot"),
        _ => (),
    };
}
