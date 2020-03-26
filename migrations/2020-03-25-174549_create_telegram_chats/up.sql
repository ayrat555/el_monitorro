CREATE TABLE telegram_chats (
    id SERIAL primary key,
    kind text NOT NULL,
    title text,
    username text,
    first_name text,
    last_name text,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE telegram_subscriptions (
   id SERIAL primary key,
   chat_id INTEGER NOT NULL references telegram_chats(id) ON DELETE CASCADE,
   feed_id INTEGER NOT NULL references feeds(id) ON DELETE CASCADE,
   created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
