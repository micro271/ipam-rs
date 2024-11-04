use super::PgRow;
use crate::models::utils::{Table, TypeTable, Updatable};
use error::RepositoryError;
use std::{
    collections::HashMap,
    {future::Future, pin::Pin},
};

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
