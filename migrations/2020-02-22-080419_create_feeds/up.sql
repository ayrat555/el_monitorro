CREATE TABLE feeds(
   id SERIAL PRIMARY KEY,
   title TEXT NOT NULL,
   link TEXT NOT NULL UNIQUE,
   description TEXT NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

ALTER TABLE feeds
      ADD CONSTRAINT feed_link_size CHECK (char_length(link) > 0);

create TABLE feed_items(
   id SERIAL PRIMARY KEY,
   feed_id INT NOT NULL references feeds(id) ON DELETE CASCADE,
   title TEXT NOT NULL,
   description TEXT NOT NULL,
   link TEXT NOT NULL,
   author TEXT NOT NULL,
   guid TEXT NOT NULL,
   categories text[] NOT NULL DEFAULT '{}',
   publication_date TIMESTAMP WITH TIME ZONE NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX feed_items_guid_index ON feed_items(feed_id, guid);
