use crate::{
    command::{reset_password_for, set_password_expire_for},
    config::get_config,
    payload::parse_payload,
};
use anyhow::Result;
use askama::Template;
use axum::{
    extract,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::net::SocketAddr;

pub async fn start_server() -> Result<()> {
    let app = Router::new().route(
        "/reset/:payload",
        get(get_reset_handler).post(post_reset_handler),
    );
    let addr = SocketAddr::from(([0, 0, 0, 0], get_config().server.port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn get_reset_handler(extract::Path(payload): extract::Path<String>) -> Response {
    let f = |payload: String| -> anyhow::Result<ResetTemplate> {
        let req = parse_payload(&payload)?;
        let config = &get_config().server;
        Ok(ResetTemplate {
            id: req.id.clone(),
            password: format!("bupt{}", req.id),
            link: format!("http://{}:{}/reset/{}", config.domain, config.port, payload),
        })
    };
    match f(payload) {
        Ok(template) => HtmlTemplate(template).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "Invalid payload: {}, contact name1e5s@bupt.edu.cn for more info.",
                e
            )),
        )
            .into_response(),
    }
}

async fn post_reset_handler(extract::Path(payload): extract::Path<String>) -> Response {
    let f = |payload: String| -> anyhow::Result<ResetPostTemplate> {
        let req = parse_payload(&payload)?;
        reset_password_for(&req.id)?;
        set_password_expire_for(&req.id)?;
        Ok(ResetPostTemplate {
            id: req.id.clone(),
            password: format!("bupt{}", req.id),
        })
    };
    match f(payload) {
        Ok(template) => HtmlTemplate(template).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "Invalid payload: {}, contact name1e5s@bupt.edu.cn for more info.",
                e
            )),
        )
            .into_response(),
    }
}

#[derive(Template)]
#[template(path = "reset.html")]
struct ResetTemplate {
    id: String,
    password: String,
    link: String,
}

#[derive(Template)]
#[template(path = "reset_post.html")]
struct ResetPostTemplate {
    id: String,
    password: String,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
