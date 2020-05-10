CREATE TABLE feeds(
   id BIGSERIAL PRIMARY KEY,
   title TEXT,
   link TEXT NOT NULL UNIQUE,
   error TEXT,
   description TEXT,
   synced_at TIMESTAMP WITH TIME ZONE,
   created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

ALTER TABLE feeds
      ADD CONSTRAINT feed_link_size CHECK (char_length(link) > 0);

create TABLE feed_items(
   id BIGSERIAL PRIMARY KEY,
   feed_id BIGINT NOT NULL references feeds(id) ON DELETE CASCADE,
   title TEXT,
   description TEXT,
   link TEXT,
   author TEXT,
   guid TEXT,
   publication_date TIMESTAMP WITH TIME ZONE NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX feed_items_title_publication_date_index ON feed_items(feed_id, title, publication_date);
CREATE INDEX feeds_synced_at_index ON feeds(synced_at);
CREATE INDEX feed_items_publication_date_index ON feed_items(publication_date);
