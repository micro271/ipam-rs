mod config;
mod database;
mod handler;
mod models;
mod services;

use axum::{
    http::Response,
    routing::{get, patch, post},
    serve, Router,
};
use config::Config;
use database::RepositoryInjection;
use handler::*;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Config { app, database } = config::Config::init().unwrap();

    let lst = tokio::net::TcpListener::bind(format!("{}:{}", app.ip, app.port)).await?;

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database.username, database.password, database.host, database.port, database.name,
    );

    let db = RepositoryInjection::new(database_url).await?;
    services::create_default_user(&db).await?;

    let db = Arc::new(db);

    let network = Router::new()
        .route("/", post(network::create))
        .route("/subnet", post(network::subnetting))
        .route(
            "/:id",
            get(network::get)
                .delete(network::delete)
                .patch(network::update),
        );

    let device = Router::new()
        .route(
            "/",
            post(device::create)
                .get(device::get)
                .patch(device::update)
                .delete(device::delete),
        )
        .route("/:network_id", post(device::create_all_devices));

    let user = Router::new()
        .route("/", post(auth::create))
        .route("/:id", patch(auth::update).delete(auth::delete));

    let location = Router::new().route(
        "/",
        get(location::get)
            .delete(location::delete)
            .patch(location::update)
            .post(location::insert),
    );

    let mount_point = Router::new().route("/", post(mount_point::insert)).route(
        "/:name",
        get(mount_point::get)
            .patch(mount_point::update)
            .delete(mount_point::delete),
    );

    let room = Router::new().route(
        "/",
        post(room::insert)
            .get(room::get)
            .patch(room::update)
            .delete(room::delete),
    );

    let vlan = Router::new().route("/", post(vlan::insert)).route(
        "/:id",
        get(vlan::get).delete(vlan::delete).patch(vlan::update),
    );

    let api_v1 = Router::new()
        .nest("/network", network)
        .nest("/device", device)
        .nest("/user", user)
        .nest("/mount_point", mount_point)
        .nest("/room", room)
        .nest("/vlan", vlan)
        .nest("/location", location);

    let app = Router::new()
        .route("/", get(hello_world))
        .nest("/api/v1", api_v1)
        .layer(axum::middleware::from_fn(auth::verify_token))
        .route("/login", post(auth::login))
        .with_state(db.clone())
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    serve(lst, app).await?;

    Ok(())
}

async fn hello_world() -> Response<String> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body("<h1>Bienvenido</h1>".to_string())
        .unwrap_or_default()
}
