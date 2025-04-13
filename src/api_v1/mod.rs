pub mod handlers;

use crate::app_state::StateType;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use self::handlers::{addresses, auth, network, node, vlan};

pub fn api_v1() -> Router<StateType> {
    let network = Router::new()
        .route("/subnet/{father}", post(network::subnetting))
        .route("/", post(network::create).get(network::get))
        .route("/{id}", delete(network::delete).patch(network::update));

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

    Router::new()
        .nest("/networks", network)
        .nest("/nodes", node)
        .nest("/users", user)
        .nest("/vlans", vlan)
        .nest("/addrs", addrs)
}
