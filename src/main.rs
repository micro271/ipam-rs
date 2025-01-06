mod database;
mod handler;
mod models;
mod services;
mod config;

use axum::{
    http::Response,
    routing::{delete, get, post, put},
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

    let Config{app, database} = config::Config::init().unwrap();

    let lst = tokio::net::TcpListener::bind(format!("{}:{}", app.ip.unwrap(), app.port.unwrap())).await?;
    
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database.username, database.password, database.host, database.port, database.name,
    );

    let db = RepositoryInjection::new(database_url).await?;
    services::create_default_user(&db).await?;

    let db = Arc::new(db);

    let network = Router::new().route("/create", put(network::create)).route(
        "/",
        get(network::get)
            .delete(network::delete)
            .patch(network::update),
    );

    let device = Router::new()
        .route("/create", put(device::create))
        .route(
            "/all/:network_id",
            get(device::get_all).put(device::create_all_devices),
        ) // create, update and get all devices
        .route("/delete", delete(device::delete))
        .route("/one", get(device::get_one).patch(device::update)); //get one device

    let user = Router::new().route("/", post(auth::create));

    let app = Router::new()
        .route("/", get(hello_world))
        .nest("/network", network)
        .nest("/device", device)
        .nest("/user", user)
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
