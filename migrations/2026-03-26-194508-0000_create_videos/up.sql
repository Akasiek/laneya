CREATE TABLE videos
(
    id            INTEGER   NOT NULL PRIMARY KEY AUTOINCREMENT,
    video_id      TEXT      NOT NULL UNIQUE,
    channel_id    INTEGER   NOT NULL REFERENCES channels (id),
    title         TEXT      NOT NULL,
    video_url     TEXT      NOT NULL,
    thumbnail_url TEXT      NOT NULL,
    published_at  TEXT      NOT NULL,
    updated_at    TEXT      NOT NULL,
    created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
