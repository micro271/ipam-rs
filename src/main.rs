mod database;
mod handler;
mod models;
mod services;

use axum::{
    http::Response,
    routing::{delete, get, post, put},
    serve, Router,
};
use database::RepositoryInjection;
use dotenv::dotenv;
use handler::*;
use std::env;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let ip = env::var("IP_ADDRESS").unwrap_or("0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());

    let lst = tokio::net::TcpListener::bind(format!("{}:{}", ip, port)).await?;

    let db_name = env::var("DB_NAME").expect("DB NAME DOESN'T DEFINED");
    let db_user = env::var("DB_USER").expect("USER DATABASE DOESN'T DEFINED");
    let db_pass = env::var("DB_PASSWD").expect("DB PASSWORD DOESN'T DEFINED");
    let db_host = env::var("DB_HOST").expect("DB HOST DOESN'T DEFINED");
    let db_port = env::var("DB_PORT").expect("DB PORT DOESN'T DEFINED");

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_pass, db_host, db_port, db_name
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
        // .layer(axum::middleware::from_fn(auth::verify_token))
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
