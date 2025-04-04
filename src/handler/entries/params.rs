use ipnet::IpNet;
use macros::MapQuery as MapQueryDerive;
use serde::Deserialize;
use std::fmt::Debug;
use uuid::Uuid;

use crate::models::network::addresses::StatusAddr;

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
