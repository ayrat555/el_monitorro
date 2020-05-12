DROP INDEX feed_items_title_publication_date_index;
CREATE UNIQUE INDEX feed_items_title_publication_date_index ON feed_items(feed_id, title, link);
