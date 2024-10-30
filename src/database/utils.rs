use super::Table;
use crate::models::utils::{TypeTable, Updatable};
use sqlx::postgres::PgRow;
use std::{boxed::Box, collections::HashMap, future::Future, pin::Pin};

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

#[derive(Debug)]
pub enum RepositoryError {
    Sqlx(String),
    RowNotFound,
    //    Unauthorized(String),
    ColumnNotFound(Option<String>),
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
