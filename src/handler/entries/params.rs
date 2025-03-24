use ipnet::IpNet;
use macros::MapQuery as MapQueryDerive;
use serde::Deserialize;
use std::fmt::Debug;
use uuid::Uuid;

use crate::models::network::addresses::StatusAddr;

#[derive(Debug, Deserialize, MapQueryDerive)]
pub struct ParamRoomStrict {
    name: String,
    address: String,
}

#[derive(Debug, Deserialize, MapQueryDerive)]
pub struct ParamRoom {
    name: Option<String>,
    address: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Subnet {
    pub father: uuid::Uuid,
    pub prefix: u8,
}

#[derive(Debug, Deserialize, MapQueryDerive)]
pub struct LocationParam {
    pub label: Option<String>,
    pub room_name: Option<String>,
    pub mount_point: Option<String>,
}

#[derive(Debug, Deserialize, MapQueryDerive)]
pub struct LocationParamStict {
    pub label: String,
    pub room_name: String,
    pub mount_point: String,
}

#[derive(Debug, Deserialize, MapQueryDerive)]
pub struct OfficeParam {
    pub id: Option<Uuid>,
    pub street: Option<String>,
    pub neighborhood: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PaginationParams {
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize, MapQueryDerive)]
pub struct ParamNetwork {
    pub network: Option<IpNet>,
    pub id: Option<Uuid>,
    pub father: Option<Uuid>,
}

#[derive(Debug, Default, MapQueryDerive, Deserialize)]
pub struct ParamAddrFilter {
    pub ip: Option<IpNet>,
    pub node_id: Option<Uuid>,
    pub status: Option<StatusAddr>,
    pub sort: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct IpNetParamNonOption {
    pub ip: IpNet,
}
