use crate::{
    database::utils::QueryResult,
    models::{self, Credential, Status},
};
use axum::{
    http::{self, Response, StatusCode},
    response::IntoResponse,
};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub network: IpNet,
    pub description: Option<String>,
    pub vlan: Option<models::Vlan>,
}

impl From<Network> for models::Network {
    fn from(value: Network) -> Self {
        let avl = 2_u32.pow(32 - value.network.prefix_len() as u32) - 2;
        Self {
            id: Uuid::new_v4(),
            network: value.network,
            description: value.description,
            available: avl,
            used: 0,
            total: 0,
            vlan: value.vlan,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Device {
    pub ip: IpAddr,
    pub description: Option<String>,
    pub office_id: Option<Uuid>,
    pub rack: Option<String>,
    pub room: Option<String>,
    pub status: Option<Status>,
    pub network_id: uuid::Uuid,
    pub credential: Option<Credential>,
}

impl From<Device> for models::Device {
    fn from(value: Device) -> Self {
        Self {
            status: Status::default(),
            ip: value.ip,
            description: value.description,
            office_id: value.office_id,
            rack: value.rack,
            room: value.room,
            network_id: value.network_id,
            credential: value.credential,
        }
    }
}

pub fn create_all_devices(network: IpNet, id: Uuid) -> Option<Vec<models::Device>> {
    let mut ips = network.hosts().collect::<Vec<IpAddr>>();
    ips.pop();
    ips.remove(0);
    let mut resp = Vec::new();
    for ip in ips {
        resp.push(models::Device {
            ip,
            description: None,
            office_id: None,
            rack: None,
            room: None,
            status: Status::default(),
            network_id: id,
            credential: None,
        });
    }

    if !resp.is_empty() {
        Some(resp)
    } else {
        None
    }
}

impl IntoResponse for QueryResult {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            QueryResult::Insert(_) => StatusCode::CREATED,
            _ => StatusCode::OK,
        };
        let body = serde_json::json!({
            "rows_affects":self.unwrap()
        })
        .to_string();

        Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(status)
            .body(body)
            .unwrap_or_default()
            .into_response()
    }
}
