﻿use crate::db::PgPool;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use http::StatusCode;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tracing::{error, info};

pub async fn run_server(
    pool: Arc<PgPool>,
    addr: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!("Запуск сервера на http://{}/", addr);

    let app = Router::new()
        .route("/", get(root))
        .route("/notify", post(notify_users))
        .with_state(pool)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new()
                        .level(tracing::Level::INFO)
                        .include_headers(true),
                )
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
        );

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .inspect_err(|e| error!("Ошибка привязки к {}: {}", addr, e))?;

    info!("Сервер успешно запущен");
    axum::serve(listener, app)
        .await
        .inspect_err(|e| error!("Ошибка сервера: {}", e))?;

    info!("Сервер остановлен");
    Ok(())
}

async fn root() -> Html<String> {
    let file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("html")
        .join("index.html");

    let page = fs::read_to_string(&file_path);
    if page.is_err() {
        let _ =
            page.inspect_err(|e| error!("Ошибка чтения по пути {}: {}", file_path.display(), e));
        return Html("<h1>Error loading HTML file</h1>".to_string());
    }

    let page = page.unwrap();
    Html(page)
}

async fn notify_users(Json(payload): Json<NotifyRequest>) -> StatusCode {
    info!("Возносится послание {}", payload.message);
    sleep(Duration::from_secs(5));
    StatusCode::OK
}

#[derive(Deserialize)]
struct NotifyRequest {
    message: String,
}
