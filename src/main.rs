use dotenv::dotenv;
use el_monitorro;
use el_monitorro::bot;
// use el_monitorro::sync::sync_job;

fn main() {
    dotenv().ok();
    env_logger::init();
    // sync_job::start_runner();
    match bot::api::start_bot() {
        Err(_) => log::error!("Couldn't start a bot"),
        _ => (),
    };

    log::info!("Started bot successfully");

    el_monitorro::rocket().launch();
}
