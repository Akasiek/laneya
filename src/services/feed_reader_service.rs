use crate::models::channel::Channel;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

const YOUTUBE_FEED_URL: &str = "https://www.youtube.com/feeds/videos.xml?channel_id=";

#[derive(Debug, Deserialize)]
#[serde(rename = "feed")]
pub struct YoutubeFeed {
    pub title: String,
    #[serde(rename = "entry", default)]
    pub entries: Vec<FeedEntry>,
}

#[derive(Debug, Deserialize)]
pub struct FeedEntry {
    #[serde(rename = "videoId")]
    pub video_id: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub title: String,
    pub link: FeedLink,
    pub published: String,
    pub updated: String,
    #[serde(rename = "group")]
    pub media_group: MediaGroup,
}

#[derive(Debug, Deserialize)]
pub struct FeedLink {
    #[serde(rename = "@href")]
    pub href: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaGroup {
    pub thumbnail: MediaThumbnail,
}

#[derive(Debug, Deserialize)]
pub struct MediaThumbnail {
    #[serde(rename = "@url")]
    pub url: String,
}

pub fn parse_feed(xml: &str) -> Result<YoutubeFeed, quick_xml::DeError> {
    quick_xml::de::from_str(xml)
}

pub async fn fetch_channel_name(channel_id: &str) -> Result<String, String> {
    let url = format!("{}{}", YOUTUBE_FEED_URL, channel_id);
    let client = Client::new();

    let xml = fetch_feed_data(&url, &client)
        .await
        .map_err(|e| format!("Could not reach YouTube: {}", e))?;

    parse_feed(&xml)
        .map(|feed| feed.title)
        .map_err(|_| format!("'{}' is not a valid YouTube channel ID.", channel_id))
}

pub async fn read_channels_feed(
    channels: Vec<Channel>,
    client: Arc<Client>,
) -> Vec<(Channel, Option<YoutubeFeed>)> {
    let mut set = tokio::task::JoinSet::new();

    for channel in channels {
        let client = Arc::clone(&client);
        set.spawn(async move {
            match read_channel_feed(&channel, &client).await {
                Ok(feed_data) => (channel, Some(feed_data)),
                Err(e) => {
                    tracing::error!("Failed to fetch feed for channel {}: {}", channel.id, e);
                    (channel, None)
                }
            }
        });
    }

    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        if let Ok(pair) = result {
            results.push(pair);
        }
    }

    results
}

pub async fn read_channel_feed(
    channel: &Channel,
    client: &Client,
) -> Result<YoutubeFeed, anyhow::Error> {
    let feed_url = create_feed_url(channel);
    let data = fetch_feed_data(&feed_url, client).await?;
    let feed = parse_feed(&data)?;
    Ok(feed)
}

pub fn create_feed_url(channel: &Channel) -> String {
    format!("{}{}", YOUTUBE_FEED_URL, channel.channel_id)
}

pub async fn fetch_feed_data(feed_url: &str, client: &Client) -> Result<String, reqwest::Error> {
    let response = client.get(feed_url).send().await?;
    let body = response.text().await?;
    Ok(body)
}
