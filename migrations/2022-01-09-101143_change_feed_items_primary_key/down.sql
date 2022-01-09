ALTER TABLE feed_items DROP CONSTRAINT feed_items_pkey;
ALTER TABLE feed_items ADD CONSTRAINT feed_items_pkey PRIMARY KEY (feed_id, title, link);
