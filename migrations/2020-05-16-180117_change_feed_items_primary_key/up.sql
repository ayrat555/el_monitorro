ALTER TABLE feed_items DROP CONSTRAINT feed_items_pkey;
ALTER TABLE feed_items ADD CONSTRAINT feed_items_pkey PRIMARY KEY (feed_id, title, link);
DROP INDEX feed_items_title_publication_date_index;
ALTER TABLE feed_items DROP COLUMN id;
