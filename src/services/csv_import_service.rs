use axum::extract::Multipart;
use axum::response::{IntoResponse, Response};
use crate::html_error;
use crate::models::channel::NewChannel;

/// Parses a Google Takeout YouTube subscriptions CSV (bytes) and returns the
/// list of channels to import.
///
/// Expected headers: `Channel ID`, `Channel URL`, `Channel title`
pub fn parse_takeout_csv(bytes: &[u8]) -> anyhow::Result<Vec<NewChannel>> {
    let mut reader = csv::Reader::from_reader(bytes);

    let headers = reader.headers()?.clone();

    let channel_id_idx = headers
        .iter()
        .position(|h| h == "Channel ID")
        .ok_or_else(|| anyhow::anyhow!(r#"Missing "Channel ID" column"#))?;

    let channel_title_idx = headers
        .iter()
        .position(|h| h == "Channel title")
        .ok_or_else(|| anyhow::anyhow!(r#"Missing "Channel title" column"#))?;

    let mut channels = Vec::new();

    for result in reader.records() {
        let record = result?;

        let channel_id = record
            .get(channel_id_idx)
            .unwrap_or("")
            .trim()
            .to_string();
        let channel_name = record
            .get(channel_title_idx)
            .unwrap_or("")
            .trim()
            .to_string();

        if channel_id.is_empty() || channel_name.is_empty() {
            continue;
        }

        channels.push(NewChannel {
            channel_id,
            channel_name,
        });
    }

    Ok(channels)
}

pub async fn extract_csv_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, Response> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name().unwrap_or("") == "csv_file" {
            return field
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| html_error!("Failed to read file: {e}").into_response());
        }
    }
    Err(html_error!("No CSV file provided.").into_response())
}