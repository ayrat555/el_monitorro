use dotenv::dotenv;

use el_monitorro::bot;
use el_monitorro::bot::deliver_job::DeliverJob;
use el_monitorro::cleaner::clean_job::CleanJob;
use el_monitorro::sync::sync_job::SyncJob;
use fang::scheduler::Scheduler;
use fang::Postgres;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let postgres = Postgres::new();

    postgres.remove_all_periodic_tasks().unwrap();

    postgres
        .push_periodic_task(&SyncJob::default(), 120)
        .unwrap();

    postgres
        .push_periodic_task(&DeliverJob::default(), 60)
        .unwrap();

    postgres
        .push_periodic_task(&CleanJob::default(), 12 * 60 * 60)
        .unwrap();

    Scheduler::start(10, 5);

    bot::api::start_bot().await;
}
