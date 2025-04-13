mod api_v1;
mod api_v2;
mod app_state;
mod config;
mod database;
mod models;
mod response;
mod services;
mod trace_layer;

use app_state::AppState;
use axum::{
    Router,
    http::{Method, header},
    routing::post,
    serve,
};
use config::Config;
use database::RepositoryInjection;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Config { app, database } = config::Config::init();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_env("LOG_LEVEL").unwrap_or(EnvFilter::new("info")))
        .init();

    let lst = tokio::net::TcpListener::bind(format!("{}:{}", app.ip, app.port)).await?;

    tracing::info!("Listening: {}:{}", app.ip, app.port);

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database.username, database.password, database.host, database.port, database.name,
    );

    let cors = app
        .allow_origin
        .map_or(CorsLayer::new().allow_origin(Any), |x| {
            CorsLayer::new()
                .allow_headers([header::CONTENT_TYPE, header::COOKIE, header::AUTHORIZATION])
                .allow_origin(x)
                .allow_credentials(true)
                .allow_methods([
                    Method::GET,
                    Method::OPTIONS,
                    Method::PATCH,
                    Method::POST,
                    Method::DELETE,
                ])
        });

    let db = RepositoryInjection::new(database_url).await?;

    services::create_default_user(&db).await?;

    let state = Arc::new(AppState::new(db, Semaphore::new(1)));

    let app = Router::new()
        .nest("/api/v1", api_v1::api_v1())
        .layer(axum::middleware::from_fn(
            api_v1::handlers::auth::verify_token,
        ))
        .route("/login", post(api_v1::handlers::auth::login))
        .with_state(Arc::clone(&state))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace_layer::make_span)
                .on_request(tower_http::trace::DefaultOnRequest::new())
                .on_response(trace_layer::on_response)
                .on_failure(tower_http::trace::DefaultOnFailure::new()),
        );

    serve(lst, app).await?;

    Ok(())
}
