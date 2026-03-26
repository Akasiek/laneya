use crate::templates::ServerErrorTemplate;
use askama::Template;
use axum::response::Html;

pub async fn server_error() -> Html<String> {
    let template = ServerErrorTemplate {};

    match template.render() {
        Ok(html) => Html(html),
        Err(_) => {
            Html("<h1>Internal Server Error</h1><p>Unable to render error page.</p>".to_string())
        }
    }
}
