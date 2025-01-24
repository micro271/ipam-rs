use super::PgRow;
use crate::models::{device::Status, user::Role};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use error::RepositoryError;
use ipnet::IpNet;
use libipam::type_net::{host_count::HostCount, vlan::VlanId};
use serde::Serialize;
use serde_json::json;
use std::{
    clone::Clone,
    collections::HashMap,
    fmt::Debug,
    net::IpAddr,
    future::Future,
};
use uuid::Uuid;

pub type ResultRepository<T> = Result<T, RepositoryError>;

pub trait Repository {
    fn get<T>(
        &self,
        primary_key: Option<HashMap<&'static str, TypeTable>>,
    ) -> impl Future<Output = ResultRepository<Vec<T>>>
    where
        T: Table + From<PgRow> + Send + Debug + Clone;
    fn insert<T>(&self, data: T) -> impl Future<Output = ResultRepository<QueryResult<T>>>
    where
        T: Table + Send + Debug + Clone;
    fn update<T, U>(
        &self,
        updater: U,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> impl Future<Output = ResultRepository<QueryResult<T>>>
    where
        T: Table + Send + Debug + Clone,
        U: Updatable + Send + Debug;
    fn delete<T>(
        &self,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> impl Future<Output = ResultRepository<QueryResult<T>>>
    where
        T: Table + Send + Debug + Clone;
}

pub trait Table {
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
                    .map(|x| format!("${}", x))
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

pub trait Updatable {
    fn get_pair(self) -> Option<HashMap<&'static str, TypeTable>>;
}

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
    }

    impl std::fmt::Display for RepositoryError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RepositoryError::Sqlx(txt) => write!(f, "Sqlx error: {}", txt),
                Self::RowNotFound => write!(f, "Row not found"),
                Self::ColumnNotFound(e) => write!(f, "The column {} is invalid", e),
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

#[derive(Debug, PartialEq)]
pub enum TypeTable {
    String(String),
    OptionUuid(Option<Uuid>),
    Uuid(Uuid),
    OptionString(Option<String>),
    Status(Status),
    Role(Role),
    OptionVlanId(Option<VlanId>),
    VlanId(VlanId),
    I64(i64),
    I32(i32),
    OptionTime(Option<time::OffsetDateTime>),
    Time(time::OffsetDateTime),
    Bool(bool),
    Null,
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
        Self::I64(*value as i64)
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

impl From<Status> for TypeTable {
    fn from(value: Status) -> Self {
        Self::Status(value)
    }
}
