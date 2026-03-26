// @generated automatically by Diesel CLI.

diesel::table! {
    channels (id) {
        id -> Integer,
        channel_id -> Text,
        channel_name -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    videos (id) {
        id -> Integer,
        video_id -> Text,
        channel_id -> Integer,
        title -> Text,
        video_url -> Text,
        thumbnail_url -> Text,
        published_at -> Text,
        updated_at -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(videos -> channels (channel_id));

diesel::allow_tables_to_appear_in_same_query!(channels, videos,);
