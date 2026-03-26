use axum::response::Html;
use crate::render_template;
use crate::templates::IndexTemplate;
use askama::Template;

pub async fn index() -> Html<String> {
    let template = IndexTemplate {};
    render_template!(template)
}
