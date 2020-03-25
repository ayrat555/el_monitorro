CREATE TABLE telegram_chats (
    id bigint primary key,
    kind text NOT NULL,
    title text,
    username text,
    first_name text,
    last_name text
);
