use askama::Template;
use crate::models::channel::Channel;
use crate::models::video::VideoResource;

#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {}

#[derive(Template)]
#[template(path = "components/video_grid.html")]
pub struct VideoGridTemplate {
    pub videos: Vec<VideoResource>,
    pub page: i64,
    pub total_pages: i64,
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
#[template(path = "components/channel/row.html")]
pub struct ChannelRowTemplate {
    pub channel: Channel,
}

#[derive(Template)]
#[template(path = "components/channel/edit_row.html")]
pub struct ChannelEditRowTemplate {
    pub channel: Channel,
    pub error: Option<String>,
}
