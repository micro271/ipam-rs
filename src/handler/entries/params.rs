use macros::MapParams as MapParamsDerive;
use serde::Deserialize;
use std::net::IpAddr;
use uuid::Uuid;

pub trait MapParams {
    fn get_pairs(
        self,
    ) -> Option<std::collections::HashMap<&'static str, crate::database::repository::TypeTable>>;
}

#[derive(Debug, Deserialize, MapParamsDerive)]
pub struct ParamRoomStrict {
    name: String,
    address: String,
}

#[derive(Debug, Deserialize, MapParamsDerive)]
pub struct ParamRoom {
    name: Option<String>,
    address: Option<String>,
}

#[derive(Debug, Deserialize, MapParamsDerive)]
pub struct ParamsDevice {
    pub ip: Option<IpAddr>,
    pub network_id: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize, MapParamsDerive)]
pub struct ParamsDeviceStrict {
    pub ip: IpAddr,
    pub network_id: uuid::Uuid,
}

#[derive(Deserialize, Debug, MapParamsDerive)]
pub struct Subnet {
    pub father: uuid::Uuid,
    pub prefix: i32,
}

#[derive(Debug, Deserialize, MapParamsDerive)]
pub struct LocationParam {
    pub label: Option<String>,
    pub room_name: Option<String>,
    pub mount_point: Option<String>,
}

#[derive(Debug, Deserialize, MapParamsDerive)]
pub struct LocationParamStict {
    pub label: String,
    pub room_name: String,
    pub mount_point: String,
}

#[derive(Debug, Deserialize, MapParamsDerive)]
pub struct OfficeParam {
    pub id: Option<Uuid>,
    pub street: Option<String>,
    pub neighborhood: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, MapParamsDerive)]
pub struct PaginationParams {
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}