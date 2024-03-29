DROP TABLE fang_periodic_tasks;
DROP TABLE fang_tasks;

CREATE TABLE fang_tasks (
     id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
     metadata jsonb NOT NULL,
     error_message TEXT,
     state fang_task_state DEFAULT 'new' NOT NULL,
     task_type VARCHAR DEFAULT 'common' NOT NULL,
     uniq_hash CHAR(64),
     scheduled_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
     created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX fang_tasks_state_index ON fang_tasks(state);
CREATE INDEX fang_tasks_type_index ON fang_tasks(task_type);
CREATE INDEX fang_tasks_scheduled_at_index ON fang_tasks(scheduled_at);
CREATE INDEX fang_tasks_uniq_hash ON fang_tasks(uniq_hash);
