mod config;
mod database;
mod handler;
mod models;
mod response;
mod services;
mod trace_layer;

use axum::{
    Router,
    http::{Method, header},
    routing::{delete, get, patch, post},
    serve,
};
use config::Config;
use database::RepositoryInjection;
use handler::{addresses, auth, network, node, vlan};
use std::sync::Arc;
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

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(trace_layer::make_span)
        .on_request(tower_http::trace::DefaultOnRequest::new())
        .on_response(trace_layer::on_response)
        .on_failure(tower_http::trace::DefaultOnFailure::new());

    let db = RepositoryInjection::new(database_url).await?;
    services::create_default_user(&db).await?;

    let db = Arc::new(db);

    let network = Router::new()
        .route("/", post(network::create).get(network::get))
        .route(
            "/{id}",
            delete(network::delete)
                .patch(network::update)
                .post(network::subnetting),
        );

    let addrs = Router::new().route("/", post(addresses::insert)).route(
        "/{network_id}",
        get(addresses::get)
            .delete(addresses::delete)
            .patch(addresses::update)
            .post(addresses::create_all_ip_addresses),
    );

    let node = Router::new().route(
        "/",
        post(node::create)
            .get(node::get)
            .patch(node::update)
            .delete(node::delete),
    );

    let user = Router::new()
        .route("/", post(auth::create))
        .route("/{id}", patch(auth::update).delete(auth::delete));

    let vlan = Router::new().route("/", post(vlan::insert)).route(
        "/{id}",
        get(vlan::get).delete(vlan::delete).patch(vlan::update),
    );

    let api_v1 = Router::new()
        .nest("/networks", network)
        .nest("/nodes", node)
        .nest("/users", user)
        .nest("/vlans", vlan)
        .nest("/addrs", addrs);

    let app = Router::new()
        .nest("/api/v1", api_v1)
        .layer(axum::middleware::from_fn(auth::verify_token))
        .route("/login", post(auth::login))
        .with_state(db.clone())
        .layer(cors)
        .layer(trace_layer);

    serve(lst, app).await?;

    Ok(())
}
