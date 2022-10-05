CREATE TABLE telegram_chats (
    id BIGSERIAL primary key,
    kind text NOT NULL,
    username text,
    first_name text,
    last_name text,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE telegram_subscriptions (
   chat_id BIGINT NOT NULL references telegram_chats(id) ON DELETE CASCADE,
   feed_id BIGINT NOT NULL references feeds(id) ON DELETE CASCADE,
   last_delivered_at TIMESTAMP WITH TIME ZONE,
   created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
   PRIMARY KEY(chat_id, feed_id)
);
