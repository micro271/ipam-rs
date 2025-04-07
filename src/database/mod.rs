pub mod repository;
pub mod sql;
pub mod transaction;

use repository::{
    MapQuery, QueryResult, Repository, ResultRepository, Table, TypeTable, Updatable,
    error::RepositoryError,
};
use sql::SqlOperations;
use sqlx::{
    Database, Pool, Postgres,
    postgres::{PgPool, PgPoolOptions, PgRow},
};
use std::{collections::HashMap, fmt::Debug};
use transaction::{BuilderPgTransaction, Transaction};

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
    async fn insert<T: Table>(&self, data: T) -> ResultRepository<QueryResult> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (data) - {:?}", data);

        let query = T::query_insert(1);
        let res = SqlOperations::insert(data, &query).execute(&self.0).await?;

        tracing::debug!("sql query result - {:?}", res);

        Ok(res.into())
    }

    async fn insert_many<T: Table>(&self, data: Vec<T>) -> ResultRepository<QueryResult> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (data) - {:?}", data);

        let len = data.len();

        tracing::trace!("1 input (data length) - {:?}", len);

        Ok(SqlOperations::insert_many(data, &T::query_insert(len))
            .execute(&self.0)
            .await?
            .into())
    }

    async fn get_one<T: Table + From<PgRow>>(
        &self,
        primary_key: impl MapQuery,
    ) -> ResultRepository<T> {
        let mut query = T::query_select();
        let query = SqlOperations::get(&mut query, primary_key, None, None);

        Ok(query.fetch_one(&self.0).await?.into())
    }

    async fn get<T: Table + From<PgRow>>(
        &self,
        column_data: impl MapQuery,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> ResultRepository<Vec<T>> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (column_data) - {:?}", column_data);
        tracing::trace!("2 input (limit) - {:?}", limit);
        tracing::trace!("3 input (offset) - {:?}", offset);

        let mut query = T::query_select();
        let query = SqlOperations::get(&mut query, column_data, limit, offset)
            .fetch_all(&self.0)
            .await?
            .into_iter()
            .map(T::from)
            .collect::<Vec<T>>();

        if query.is_empty() {
            Err(RepositoryError::RowNotFound)
        } else {
            Ok(query)
        }
    }

    async fn update<T: Table, U: Updatable>(
        &self,
        updater: U,
        condition: impl MapQuery,
    ) -> ResultRepository<QueryResult> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (updater) - {:?}", updater);
        tracing::trace!("2 input (condition) - {:?}", condition);

        let mut query = T::query_update();
        let condition = condition.get_pairs();
        let result = SqlOperations::update(
            updater.get_pair().ok_or(RepositoryError::UpdaterEmpty)?,
            condition,
            &mut query,
        )
        .execute(&self.0)
        .await?;

        tracing::debug!("sql query result - {:?}", result);

        Ok(result.into())
    }

    async fn delete<T: Table>(&self, condition: impl MapQuery) -> ResultRepository<QueryResult> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (condition) - {:?}", condition);

        let condition = condition.get_pairs();
        let mut query = T::query_delete();
        let res = SqlOperations::delete(condition, &mut query)
            .execute(&self.0)
            .await?;

        tracing::debug!("sql operation result - {:?}", res);

        Ok(res.into())
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

impl Transaction for RepositoryInjection<Postgres> {
    async fn transaction(&self) -> Result<BuilderPgTransaction<'_>, RepositoryError> {
        Ok(BuilderPgTransaction::new(self.0.begin().await?))
    }
}

impl Updatable for HashMap<&'static str, TypeTable> {
    fn get_pair(self) -> Option<HashMap<&'static str, TypeTable>> {
        Some(self)
    }
}
