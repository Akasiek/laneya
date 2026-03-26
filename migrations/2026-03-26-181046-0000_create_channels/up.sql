-- Saves youtube channel ids for fetching rss feed info
CREATE TABLE channels
(
    id           INTEGER   NOT NULL PRIMARY KEY AUTOINCREMENT,
    channel_id   TEXT      NOT NULL,
    channel_name TEXT      NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
