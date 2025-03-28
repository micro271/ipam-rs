pub mod repository;
pub mod sql;
pub mod transaction;

use futures::stream::StreamExt;
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
    async fn insert<T: Table>(&self, data: T) -> ResultRepository<QueryResult<T>> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (data) - {:?}", data);

        let query = T::query_insert();
        let res = SqlOperations::insert(data, &query).execute(&self.0).await?;

        tracing::debug!("sql query result - {:?}", res);

        Ok(QueryResult::Insert(res.rows_affected()))
    }

    async fn get<T: Table + From<PgRow>>(
        &self,
        column_data: impl MapQuery,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> ResultRepository<QueryResult<T>> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (column_data) - {:?}", column_data);
        tracing::trace!("2 input (limit) - {:?}", limit);
        tracing::trace!("3 input (offset) - {:?}", offset);

        let mut query = T::query_select();
        let mut vec_resp = Vec::new();
        let mut query = SqlOperations::get(&mut query, column_data, limit, offset).fetch(&self.0);

        while let Some(Ok(e)) = query.next().await {
            vec_resp.push(T::from(e));
        }

        tracing::debug!("sql query result - {:?}", vec_resp);

        if vec_resp.is_empty() {
            Err(RepositoryError::RowNotFound)
        } else {
            Ok(QueryResult::Select {
                data: vec_resp,
                offset,
                limit,
            })
        }
    }

    async fn update<T: Table, U: Updatable>(
        &self,
        updater: U,
        condition: impl MapQuery,
    ) -> ResultRepository<QueryResult<T>> {
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

        Ok(QueryResult::Update(result.rows_affected()))
    }

    async fn delete<T: Table>(&self, condition: impl MapQuery) -> ResultRepository<QueryResult<T>> {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (condition) - {:?}", condition);

        let condition = condition.get_pairs();
        let mut query = T::query_delete();
        let res = SqlOperations::delete(condition, &mut query)
            .execute(&self.0)
            .await?;

        tracing::debug!("sql operation result - {:?}", res);

        Ok(QueryResult::Delete(res.rows_affected()))
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
