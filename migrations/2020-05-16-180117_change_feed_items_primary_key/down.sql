CREATE UNIQUE INDEX feed_items_title_publication_date_index ON feed_items(feed_id, title, link);
ALTER TABLE feed_items DROP CONSTRAINT feed_items_pkey;
ALTER TABLE feed_items ADD COLUMN id BIGSERIAL PRIMARY KEY;
