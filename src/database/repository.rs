use super::PgRow;
use ipnet::IpNet;
use error::RepositoryError;
use std::{
    net::IpAddr,
    collections::HashMap,
    {future::Future, pin::Pin},
};
use uuid::Uuid;
use crate::models::{user::Role, device::{Credential, Status}, network::Vlan};


pub type ResultRepository<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + 'a + Send>>;

pub trait Repository {
    fn get<'a, T>(
        &'a self,
        primary_key: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, Vec<T>>
    where
        T: Table + From<PgRow> + 'a + Send;
    fn insert<'a, T>(&'a self, data: Vec<T>) -> ResultRepository<'a, QueryResult>
    where
        T: Table + 'a + Send;
    fn update<'a, T, U>(
        &'a self,
        updater: U,
        condition: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, QueryResult>
    where
        T: Table + 'a + Send,
        U: Updatable<'a> + Send + 'a;
    fn delete<'a, T>(
        &'a self,
        condition: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, QueryResult>
    where
        T: Table + 'a + Send;
}

pub trait Table {
    fn name() -> String;
    fn query_insert() -> String;
    fn get_fields(self) -> Vec<TypeTable>;
    fn columns() -> Vec<&'static str>;
}

pub trait Updatable<'a> {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>>;
}

pub enum QueryResult {
    Insert(u64),
    Update(u64),
    Delete(u64),
}

impl QueryResult {
    pub fn unwrap(self) -> u64 {
        match self {
            QueryResult::Insert(e) => e,
            QueryResult::Update(e) => e,
            QueryResult::Delete(e) => e,
        }
    }
}

pub mod error {

    #[derive(Debug)]
    pub enum RepositoryError {
        Sqlx(String),
        RowNotFound,
        //    Unauthorized(String),
        ColumnNotFound(Option<String>),
    }

    impl std::fmt::Display for RepositoryError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RepositoryError::Sqlx(txt) => write!(f, "Sqlx error: {}", txt),
                Self::RowNotFound => write!(f, "Row doesn't exist"),
                Self::ColumnNotFound(e) => match e {
                    Some(e) => {
                        write!(f, "The column {} didn't find", e)
                    }
                    None => {
                        write!(f, "Undefined collumn")
                    }
                },
            }
        }
    }

    impl std::error::Error for RepositoryError {}

    impl From<sqlx::Error> for RepositoryError {
        fn from(value: sqlx::Error) -> Self {
            match value {
                sqlx::Error::RowNotFound => Self::RowNotFound,
                sqlx::Error::ColumnNotFound(e) => Self::ColumnNotFound(Some(e)),
                e => Self::Sqlx(e.to_string()),
            }
        }
    }
}


#[derive(Debug)]
pub enum TypeTable {
    String(String),
    OptionUuid(Option<Uuid>),
    Uuid(Uuid),
    OptionString(Option<String>),
    Status(Status),
    Int32(i32),
    Role(Role),
    Float64(f64),
    OptionVlan(Option<i32>),
    OptionCredential(Option<Credential>),
}

impl From<Option<Vlan>> for TypeTable {
    fn from(value: Option<Vlan>) -> Self {
        Self::OptionVlan(value.map(|vlan| *vlan as i32))
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

impl From<u8> for TypeTable {
    fn from(value: u8) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<u16> for TypeTable {
    fn from(value: u16) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<u32> for TypeTable {
    fn from(value: u32) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<i8> for TypeTable {
    fn from(value: i8) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<i16> for TypeTable {
    fn from(value: i16) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<Option<Credential>> for TypeTable {
    fn from(value: Option<Credential>) -> Self {
        Self::OptionCredential(value)
    }
}

impl From<i32> for TypeTable {
    fn from(value: i32) -> Self {
        Self::Int32(value)
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

impl From<f32> for TypeTable {
    fn from(value: f32) -> Self {
        Self::Float64(value as f64)
    }
}
