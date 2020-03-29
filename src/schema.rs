table! {
    feed_items (id) {
        id -> Int8,
        feed_id -> Int8,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        link -> Nullable<Text>,
        author -> Nullable<Text>,
        guid -> Nullable<Text>,
        publication_date -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    feeds (id) {
        id -> Int8,
        title -> Nullable<Text>,
        link -> Text,
        error -> Nullable<Text>,
        description -> Nullable<Text>,
        synced_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    tasks (id) {
        id -> Int8,
        job_uuid -> Text,
        status -> Text,
        result -> Nullable<Text>,
        run_at -> Nullable<Int8>,
        queue -> Nullable<Text>,
        attempts -> Int4,
        max_attempts -> Int4,
        created_at -> Int8,
        updated_at -> Int8,
        cron -> Nullable<Text>,
        interval -> Nullable<Int8>,
        job -> Text,
    }
}

table! {
    telegram_chats (id) {
        id -> Int8,
        kind -> Text,
        title -> Nullable<Text>,
        username -> Nullable<Text>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    telegram_subscriptions (chat_id, feed_id) {
        chat_id -> Int8,
        feed_id -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(feed_items -> feeds (feed_id));
joinable!(telegram_subscriptions -> feeds (feed_id));
joinable!(telegram_subscriptions -> telegram_chats (chat_id));

allow_tables_to_appear_in_same_query!(
    feed_items,
    feeds,
    tasks,
    telegram_chats,
    telegram_subscriptions,
);
