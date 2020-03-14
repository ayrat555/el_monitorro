use super::job::{JobProcessor, DEFAULT_QUEUE};
use actix::System;
use background_jobs::memory_storage::Storage;
use background_jobs::{QueueHandle, ServerConfig, WorkerConfig};
use once_cell::sync::OnceCell;

pub fn job_queue() -> &'static QueueHandle {
    static JOB_QUEUE: OnceCell<QueueHandle> = OnceCell::new();
    JOB_QUEUE.get_or_init(|| init_job_queue())
}

fn init_job_queue() -> QueueHandle {
    let _sys = System::new("actix-system");
    let storage = Storage::new();
    let queue_handle = ServerConfig::new(storage).thread_count(8).start();

    WorkerConfig::new(move || ())
        .register(JobProcessor)
        .set_processor_count(DEFAULT_QUEUE, 4)
        .start(queue_handle.clone());

    queue_handle
}
