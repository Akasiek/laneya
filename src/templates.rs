use askama::Template;
use crate::models::channel::Channel;
use crate::models::video::Video;

#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {}

#[derive(Template)]
#[template(path = "pages/video_grid.html")]
pub struct VideoGridTemplate {
    pub videos: Vec<Video>,
}

#[derive(Template)]
#[template(path = "pages/server_error.html")]
pub struct ServerErrorTemplate {}

#[derive(Template)]
#[template(path = "pages/channels.html")]
pub struct ChannelsTemplate {
    pub channels: Vec<Channel>,
}

#[derive(Template)]
#[template(path = "pages/channel_row.html")]
pub struct ChannelRowTemplate {
    pub channel: Channel,
}

#[derive(Template)]
#[template(path = "pages/channel_edit_row.html")]
pub struct ChannelEditRowTemplate {
    pub channel: Channel,
    pub error: Option<String>,
}
