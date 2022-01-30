table! {
    backup_feeds (id) {
        id -> Int4,
        feed_id -> Nullable<Int4>,
        url -> Varchar,
    }
}

table! {
    feeds (id) {
        id -> Int4,
        canvas_id -> Varchar,
        url -> Varchar,
        last_update -> Timestamp,
    }
}

table! {
    subscriptions (id) {
        id -> Int4,
        server_id -> Varchar,
        channel_id -> Varchar,
        feed_id -> Nullable<Int4>,
    }
}

joinable!(backup_feeds -> feeds (feed_id));
joinable!(subscriptions -> feeds (feed_id));

allow_tables_to_appear_in_same_query!(
    backup_feeds,
    feeds,
    subscriptions,
);
