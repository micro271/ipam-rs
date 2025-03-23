use super::PgRow;
use crate::{
    MapQuery,
    models::{
        network::{self, Kind, StatusNetwork, addresses::StatusAddr},
        user::Role,
    },
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use error::RepositoryError;
use ipnet::IpNet;
use libipam::types::{host_count::HostCount, vlan::VlanId};
use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, fmt::Debug, net::IpAddr};
use uuid::Uuid;

pub type ResultRepository<T> = Result<T, RepositoryError>;

pub trait Repository {
    fn get<T: Table + From<PgRow>>(
        &self,
        primary_key: impl MapQuery,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> impl Future<Output = ResultRepository<Vec<T>>>;

    fn insert<T: Table>(&self, data: T) -> impl Future<Output = ResultRepository<QueryResult<T>>>;

    fn update<T: Table, U: Updatable>(
        &self,
        updater: U,
        condition: impl MapQuery,
    ) -> impl Future<Output = ResultRepository<QueryResult<T>>>;

    fn delete<T: Table>(
        &self,
        condition: impl MapQuery,
    ) -> impl Future<Output = ResultRepository<QueryResult<T>>>;
}

pub trait Table: Send + Sync + Debug {
    fn name() -> String;

    fn query_insert() -> String
    where
        Self: Table,
    {
        let columns = Self::columns();
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::name(),
            { columns.join(", ") },
            {
                (1..=columns.len())
                    .map(|x| format!("${x}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            },
        )
    }

    fn get_fields(self) -> Vec<TypeTable>;

    fn columns() -> Vec<&'static str>;

    fn query_update() -> String
    where
        Self: Table,
    {
        format!("UPDATE {} SET", Self::name())
    }

    fn query_delete() -> String
    where
        Self: Table,
    {
        format!("DELETE FROM {}", Self::name())
    }
}

pub trait Updatable: Sync + Send + Debug {
    fn get_pair(self) -> Option<HashMap<&'static str, TypeTable>>;
}

#[derive(Debug)]
pub enum QueryResult<T> {
    Insert(u64),
    Update(u64),
    Delete(u64),
    Select(Vec<T>),
}

impl<S> IntoResponse for QueryResult<S>
where
    S: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let (body, status) = match self {
            Self::Insert(e) => (
                json!({
                    "status": 201,
                    "row_inserted": e,
                    "success": true,
                }),
                StatusCode::CREATED,
            ),
            Self::Update(e) => (
                json!({
                    "status": 200,
                    "row_updated": e,
                    "success": true,
                }),
                StatusCode::OK,
            ),
            Self::Delete(e) => (
                json!({
                    "status": 200,
                    "row_deleted": e,
                    "success": true,
                }),
                StatusCode::OK,
            ),
            Self::Select(elements) => (
                json!({
                    "status": 200,
                    "length": elements.len(),
                    "data": elements,
                    "success": true,
                }),
                StatusCode::OK,
            ),
        };

        Response::builder()
            .status(status)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body::<String>(body.to_string())
            .unwrap_or_default()
            .into_response()
    }
}

impl<T> From<Vec<T>> for QueryResult<T>
where
    T: Table + Serialize,
{
    fn from(value: Vec<T>) -> Self {
        Self::Select(value)
    }
}

pub mod error {
    #[derive(Debug)]
    pub enum RepositoryError {
        Sqlx(String),
        RowNotFound,
        ColumnNotFound(String),
        UpdaterEmpty,
    }

    impl std::fmt::Display for RepositoryError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RepositoryError::Sqlx(txt) => write!(f, "Sqlx error: {txt}"),
                Self::RowNotFound => write!(f, "Row not found"),
                Self::ColumnNotFound(e) => write!(f, "The column {e} is invalid"),
                Self::UpdaterEmpty => write!(f, "There isn't element to change"),
            }
        }
    }

    impl std::error::Error for RepositoryError {}

    impl From<sqlx::Error> for RepositoryError {
        fn from(value: sqlx::Error) -> Self {
            match value {
                sqlx::Error::ColumnNotFound(e) => Self::ColumnNotFound(e),
                e => Self::Sqlx(e.to_string()),
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeTable {
    String(String),
    OptionUuid(Option<Uuid>),
    Uuid(Uuid),
    OptionString(Option<String>),
    StatusAddr(StatusAddr),
    StatusNetwork(network::StatusNetwork),
    Role(Role),
    OptionVlanId(Option<VlanId>),
    VlanId(VlanId),
    I32(i32),
    HostCount(HostCount),
    OptionTime(Option<time::OffsetDateTime>),
    Time(time::OffsetDateTime),
    Bool(bool),
    Kind(Kind),
    Null,
}

#[macro_export]
macro_rules! bind_query {
    ($query:expr_2021, $value:expr_2021) => {
        match $value {
            TypeTable::OptionUuid(e) => $query.bind(e),
            TypeTable::Kind(e) => $query.bind(e),
            TypeTable::Uuid(e) => $query.bind(e),
            TypeTable::String(s) => $query.bind(s),
            TypeTable::OptionString(opt) => $query.bind(opt),
            TypeTable::StatusAddr(status) => $query.bind(status),
            TypeTable::Role(role) => $query.bind(role),
            TypeTable::OptionVlanId(e) => $query.bind(e),
            TypeTable::Bool(e) => $query.bind(e),
            TypeTable::OptionTime(e) => $query.bind(e),
            TypeTable::Time(e) => $query.bind(e),
            TypeTable::VlanId(e) => $query.bind(e),
            TypeTable::HostCount(e) => $query.bind(e),
            TypeTable::I32(e) => $query.bind(e),
            TypeTable::StatusNetwork(e) => $query.bind(e),
            TypeTable::Null => $query,
        }
    };
}

impl From<StatusNetwork> for TypeTable {
    fn from(value: StatusNetwork) -> Self {
        Self::StatusNetwork(value)
    }
}

impl From<Kind> for TypeTable {
    fn from(value: Kind) -> Self {
        Self::Kind(value)
    }
}

impl From<bool> for TypeTable {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<time::OffsetDateTime> for TypeTable {
    fn from(value: time::OffsetDateTime) -> Self {
        Self::Time(value)
    }
}

impl From<Option<time::OffsetDateTime>> for TypeTable {
    fn from(value: Option<time::OffsetDateTime>) -> Self {
        Self::OptionTime(value)
    }
}

impl From<VlanId> for TypeTable {
    fn from(value: VlanId) -> Self {
        Self::VlanId(value)
    }
}

impl From<i32> for TypeTable {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<Option<VlanId>> for TypeTable {
    fn from(value: Option<VlanId>) -> Self {
        Self::OptionVlanId(value.filter(|x| !(..1).contains(&**x)))
    }
}

impl From<HostCount> for TypeTable {
    fn from(value: HostCount) -> Self {
        Self::HostCount(value)
    }
}

impl From<Uuid> for TypeTable {
    fn from(value: Uuid) -> Self {
        TypeTable::Uuid(value)
    }
}

impl From<Role> for TypeTable {
    fn from(value: Role) -> Self {
        Self::Role(value)
    }
}

impl From<Option<Uuid>> for TypeTable {
    fn from(value: Option<Uuid>) -> Self {
        Self::OptionUuid(value)
    }
}

impl From<IpAddr> for TypeTable {
    fn from(value: IpAddr) -> Self {
        Self::String(value.to_string())
    }
}

impl From<IpNet> for TypeTable {
    fn from(value: IpNet) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for TypeTable {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Option<String>> for TypeTable {
    fn from(value: Option<String>) -> Self {
        Self::OptionString(value.filter(|x| !x.is_empty()))
    }
}

impl From<StatusAddr> for TypeTable {
    fn from(value: StatusAddr) -> Self {
        Self::StatusAddr(value)
    }
}
