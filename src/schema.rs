table! {
    feed_items (id) {
        id -> Int4,
        feed_id -> Int4,
        title -> Text,
        description -> Text,
        link -> Text,
        author -> Text,
        guid -> Text,
        categories -> Array<Text>,
        publication_date -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    feeds (id) {
        id -> Int4,
        title -> Text,
        link -> Text,
        error -> Nullable<Text>,
        description -> Text,
        synced_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(feed_items -> feeds (feed_id));

allow_tables_to_appear_in_same_query!(
    feed_items,
    feeds,
);
