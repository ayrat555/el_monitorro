ALTER TABLE feed_items ADD COLUMN content_hash VARCHAR(10) NOT NULL GENERATED ALWAYS AS (encode(sha256((link || title || description)::bytea), 'hex')) STORED;
