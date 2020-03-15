CREATE TABLE tasks (
    id bigserial primary key,
    job_uuid text NOT NULL,
    status text NOT NULL,
    result text,
    run_at bigint,
    queue text,
    attempts integer NOT NULL,
    max_attempts integer NOT NULL,
    created_at bigint NOT NULL,
    updated_at bigint NOT NULL,
    cron text,
    "interval" bigint,
    job text NOT NULL
);
