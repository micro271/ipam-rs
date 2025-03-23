use ipnet::IpNet;
use macros::MapQuery as MapQueryDerive;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug, net::IpAddr};
use uuid::Uuid;

use crate::database::repository::TypeTable;

pub trait MapQuery: Debug + Send + Sync {
    fn get_pairs(
        self,
    ) -> Option<std::collections::HashMap<&'static str, crate::database::repository::TypeTable>>;
}

impl MapQuery for Option<HashMap<&'static str, TypeTable>> {
    fn get_pairs(
        self,
    ) -> Option<std::collections::HashMap<&'static str, crate::database::repository::TypeTable>>
    {
        self
    }
}

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

#[derive(Debug, Deserialize, MapQueryDerive)]
pub struct ParamsDevice {
    pub ip: Option<IpAddr>,
    pub network_id: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize, MapQueryDerive, Copy, Clone)]
pub struct ParamsDeviceStrict {
    pub ip: IpAddr,
    pub network_id: uuid::Uuid,
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
pub struct ParamAddresse {
    pub hostname: Option<String>,
    pub ip: Option<IpNet>,
    pub network_id: Option<Uuid>,
}

#[derive(Debug, Default, MapQueryDerive, Deserialize)]
pub struct ParamAddresseStrict {
    pub ip: IpNet,
    pub network_id: Uuid,
}
