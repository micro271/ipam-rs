pub mod repository;
pub mod sql;
pub mod transaction;

use futures::stream::StreamExt;
use repository::{
    error::RepositoryError, QueryResult, Repository, ResultRepository, Table, TypeTable, Updatable,
};
use sql::SqlOperations;
use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Database, Pool, Postgres,
};
use std::{clone::Clone, collections::HashMap, fmt::Debug};
use transaction::{BuilderPgTransaction, Transaction};

use crate::MapQuery;
#[derive(Debug)]
pub struct RepositoryInjection<DB>(Pool<DB>)
where
    DB: Database;

impl RepositoryInjection<Postgres> {
    pub async fn new(url: String) -> Result<Self, RepositoryError> {
        Ok(Self(
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await?,
        ))
    }
}

impl Repository for RepositoryInjection<Postgres> {
    async fn insert<T>(&self, data: T) -> ResultRepository<QueryResult<T>>
    where
        T: Table + Send + Debug + Clone,
    {
        let query = T::query_insert();
        let mut tmp = sqlx::query(&query);
        let data = data.get_fields();
        for i in data {
            tmp = match i {
                TypeTable::String(s) => tmp.bind(s),
                TypeTable::OptionString(opt) => tmp.bind(opt),
                TypeTable::Status(status) => tmp.bind(status),
                TypeTable::Uuid(e) => tmp.bind(e),
                TypeTable::Role(r) => tmp.bind(r),
                TypeTable::OptionUuid(e) => tmp.bind(e),
                TypeTable::Bool(e) => tmp.bind(e),
                TypeTable::OptionTime(e) => tmp.bind(e),
                TypeTable::Time(e) => tmp.bind(e),
                TypeTable::Null => tmp,
                TypeTable::I64(e) => tmp.bind(e),
                TypeTable::OptionVlanId(e) => tmp.bind(e),
                TypeTable::VlanId(e) => tmp.bind(e),
                TypeTable::I32(e) => tmp.bind(e),
            };
        }
        Ok(QueryResult::Insert(
            tmp.execute(&self.0).await?.rows_affected(),
        ))
    }

    async fn get<T>(
        &self,
        column_data: impl MapQuery,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> ResultRepository<Vec<T>>
    where
        T: Table + From<PgRow> + Send + Debug,
    {
        let mut query = format!("SELECT * FROM {}", T::name());
        let mut vec_resp = Vec::new();
        let query = SqlOperations::get(&mut query, column_data, limit, offset);
        let mut fetch = query.fetch(&self.0);
        while let Some(Ok(e)) = fetch.next().await {
            vec_resp.push(T::from(e));
        }
        if !vec_resp.is_empty() {
            Ok(vec_resp)
        } else {
            Err(RepositoryError::RowNotFound)
        }
    }

    async fn update<T, U>(
        &self,
        updater: U,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> ResultRepository<QueryResult<T>>
    where
        T: Table + Send + Debug,
        U: Updatable + Send + Debug,
    {
        tracing::debug!(
            "Update element % new_data: {:?} - condition {:?} %",
            updater,
            condition
        );
        if let Some(pair) = updater.get_pair() {
            let cols = T::columns();

            let mut query = T::query_update();

            let mut pos_values = HashMap::new();

            let mut pos = 1;
            let len = pair.len();
            for i in pair.keys() {
                if !cols.contains(i) {
                    return Err(RepositoryError::ColumnNotFound(i.to_string()));
                }

                query.push_str(&format!(" {} = ${}", i, pos));
                pos_values.insert(pos, pair.get(i).unwrap());
                if len > pos {
                    query.push(',');
                }
                pos += 1
            }

            let condition = match condition {
                Some(e) => {
                    query.push_str(" WHERE");
                    e
                }
                None => HashMap::new(),
            };

            let len = condition.len() + pos - 1;
            for i in condition.keys() {
                pos_values.insert(pos, condition.get(i).unwrap());
                query.push_str(&format!(" {} = ${}", i, pos));
                if pos < len {
                    query.push_str(" AND");
                }
                pos += 1;
            }

            let mut sql = sqlx::query(&query);
            for i in 1..pos {
                sql = match pos_values.get(&i).unwrap() {
                    TypeTable::OptionVlanId(e) => sql.bind(e),
                    TypeTable::VlanId(e) => sql.bind(e),
                    TypeTable::String(s) => sql.bind(s),
                    TypeTable::OptionString(value) => sql.bind(value),
                    TypeTable::Status(value) => sql.bind(value),
                    TypeTable::Uuid(e) => sql.bind(e),
                    TypeTable::Role(value) => sql.bind(value),
                    TypeTable::OptionUuid(e) => sql.bind(e),
                    TypeTable::Bool(e) => sql.bind(e),
                    TypeTable::OptionTime(e) => sql.bind(e),
                    TypeTable::Time(e) => sql.bind(e),
                    TypeTable::Null => sql,
                    TypeTable::I64(e) => sql.bind(e),
                    TypeTable::I32(e) => sql.bind(e),
                };
            }

            match sql.execute(&self.0).await {
                Ok(e) => Ok(QueryResult::Update(e.rows_affected())),
                Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
            }
        } else {
            Err(RepositoryError::ColumnNotFound("".to_string()))
        }
    }

    async fn delete<T>(
        &self,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> ResultRepository<QueryResult<T>>
    where
        T: Table + Send + Debug,
    {
        let mut query = T::query_delete();

        match condition {
            Some(condition) if !condition.is_empty() => {
                let columns = T::columns();

                query.push_str(" WHERE");

                let mut pos_column = HashMap::new();
                let mut pos = 1;

                let len = condition.len();
                for t in condition.keys() {
                    if !columns.contains(t) {
                        return Err(RepositoryError::ColumnNotFound(t.to_string()));
                    }

                    query.push_str(&format!(" {} = ${}", t, pos));
                    pos_column.insert(pos, condition.get(t).unwrap());
                    if pos < len {
                        query.push_str(" AND");
                    }
                    pos += 1;
                }

                let mut ex = sqlx::query(&query);

                for i in 1..pos {
                    ex = match pos_column.get(&i).unwrap() {
                        TypeTable::OptionVlanId(e) => ex.bind(e),
                        TypeTable::VlanId(e) => ex.bind(e),
                        TypeTable::OptionUuid(e) => ex.bind(e),
                        TypeTable::String(s) => ex.bind(s),
                        TypeTable::OptionString(s) => ex.bind(s),
                        TypeTable::Uuid(e) => ex.bind(e),
                        TypeTable::Status(status) => ex.bind(status),
                        TypeTable::Role(role) => ex.bind(role),
                        TypeTable::Bool(e) => ex.bind(e),
                        TypeTable::OptionTime(e) => ex.bind(e),
                        TypeTable::Time(e) => ex.bind(e),
                        TypeTable::I64(e) => ex.bind(e),
                        TypeTable::I32(e) => ex.bind(e),
                        TypeTable::Null => ex,
                    };
                }

                match ex.execute(&self.0).await {
                    Ok(e) => Ok(QueryResult::Delete(e.rows_affected())),
                    Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
                }
            }

            None => match sqlx::query(&query).execute(&self.0).await {
                Ok(e) => Ok(QueryResult::Delete(e.rows_affected())),
                Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
            },
            _ => Err(RepositoryError::ColumnNotFound("".to_string())),
        }
    }
}

impl std::ops::Deref for RepositoryInjection<Postgres> {
    type Target = PgPool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RepositoryInjection<Postgres> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> Transaction<'a> for RepositoryInjection<Postgres> {
    fn transaction(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<BuilderPgTransaction<'a>, RepositoryError>>
                + '_
                + Send,
        >,
    > {
        Box::pin(async { Ok(BuilderPgTransaction::new(self.0.begin().await?)) })
    }
}

impl Updatable for HashMap<&'static str, TypeTable> {
    fn get_pair(self) -> Option<HashMap<&'static str, TypeTable>> {
        Some(self)
    }
}
