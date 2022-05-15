table! {
    feed_items (feed_id, content_hash) {
        feed_id -> Int8,
        title -> Text,
        description -> Nullable<Text>,
        link -> Text,
        author -> Nullable<Text>,
        guid -> Nullable<Text>,
        publication_date -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        content_hash -> Bpchar,
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
        feed_type -> Text,
        sync_retries -> Int4,
        sync_skips -> Int4,
        content_fields -> Nullable<Array<Text>>,
    }
}

table! {
    telegram_chats (id) {
        id -> Int8,
        kind -> Text,
        username -> Nullable<Text>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        title -> Nullable<Text>,
        utc_offset_minutes -> Nullable<Int4>,
        template -> Nullable<Text>,
        filter_words -> Nullable<Array<Text>>,
    }
}

table! {
    telegram_subscriptions (chat_id, feed_id) {
        chat_id -> Int8,
        feed_id -> Int8,
        last_delivered_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        template -> Nullable<Text>,
        filter_words -> Nullable<Array<Text>>,
        has_updates -> Bool,
    }
}

joinable!(feed_items -> feeds (feed_id));
joinable!(telegram_subscriptions -> feeds (feed_id));
joinable!(telegram_subscriptions -> telegram_chats (chat_id));

allow_tables_to_appear_in_same_query!(feed_items, feeds, telegram_chats, telegram_subscriptions,);
