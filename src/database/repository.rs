use super::PgRow;
use crate::models::{
    device::Status,
    user::Role,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use error::RepositoryError;
use ipnet::IpNet;
use libipam::type_net::{host_count::HostCount, vlan::Vlan};
use serde::Serialize;
use serde_json::json;
use std::{
    clone::Clone,
    collections::HashMap,
    fmt::Debug,
    net::IpAddr,
    {future::Future, pin::Pin},
};
use uuid::Uuid;

pub type ResultRepository<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + 'a + Send>>;

pub trait Repository {
    fn get<'a, T>(
        &'a self,
        primary_key: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, Vec<T>>
    where
        T: Table + From<PgRow> + 'a + Send + Debug + Clone;
    fn insert<'a, T>(&'a self, data: Vec<T>) -> ResultRepository<'a, QueryResult<T>>
    where
        T: Table + 'a + Send + Debug + Clone;
    fn update<'a, T, U>(
        &'a self,
        updater: U,
        condition: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, QueryResult<T>>
    where
        T: Table + 'a + Send + Debug + Clone,
        U: Updatable<'a> + Send + 'a + Debug;
    fn delete<'a, T>(
        &'a self,
        condition: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, QueryResult<T>>
    where
        T: Table + 'a + Send + Debug + Clone;
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

pub trait Updatable<'a> {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>>;
}

pub enum QueryResult<T> {
    Insert { row_affect: u64, data: Vec<T> },
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
            Self::Insert { row_affect, data } => (
                json!({
                    "status": 201,
                    "row_affect": row_affect,
                    "data": data
                }),
                StatusCode::CREATED,
            ),
            Self::Update(e) | Self::Delete(e) => (
                json!({
                    "status": 200,
                    "row_affect": e
                }),
                StatusCode::OK,
            ),
            Self::Select(elements) => (
                json!({
                    "status": 200,
                    "length": elements.len(),
                    "data": elements,
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
    OptionVlan(Option<i32>),
    I64(i64),
    Null,
}

impl From<Option<Vlan>> for TypeTable {
    fn from(value: Option<Vlan>) -> Self {
        Self::OptionVlan(value.map(|vlan| *vlan as i32))
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
        Self::OptionString(value)
    }
}

impl From<Status> for TypeTable {
    fn from(value: Status) -> Self {
        Self::Status(value)
    }
}
