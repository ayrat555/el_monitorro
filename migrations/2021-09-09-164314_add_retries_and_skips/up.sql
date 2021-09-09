ALTER TABLE feeds ADD COLUMN sync_retries INTEGER NOT NULL DEFAULT 0;
ALTER TABLE feeds ADD COLUMN sync_skips INTEGER NOT NULL DEFAULT 0;

CREATE INDEX feeds_sync_retries_index ON feeds(sync_retries);
CREATE INDEX feeds_sync_skips_index ON feeds(sync_skips);
