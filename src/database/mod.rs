pub mod repository;
pub mod sql;
pub mod transaction;

use crate::MapQuery;
use futures::stream::StreamExt;
use repository::{
    error::RepositoryError, QueryResult, Repository, ResultRepository, Table, TypeTable, Updatable,
};
use sql::SqlOperations;
use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Database, Pool, Postgres,
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
    async fn insert<T>(&self, data: T) -> ResultRepository<QueryResult<T>>
    where
        T: Table + Debug,
    {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (data) - {:?}", data);

        let query = T::query_insert();
        let res = SqlOperations::insert(data, &query).execute(&self.0).await?;

        tracing::debug!("sql query result - {:?}", res);

        Ok(QueryResult::Insert(res.rows_affected()))
    }

    async fn get<T>(
        &self,
        column_data: impl MapQuery,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> ResultRepository<Vec<T>>
    where
        T: Table + From<PgRow> + Debug,
    {
        tracing::trace!("REPOSITORY");
        tracing::trace!("1 input (column_data) - {:?}", column_data);
        tracing::trace!("2 input (limit) - {:?}", limit);
        tracing::trace!("3 input (offset) - {:?}", offset);

        let mut query = format!("SELECT * FROM {}", T::name());
        let mut vec_resp = Vec::new();
        let mut query = SqlOperations::get(&mut query, column_data, limit, offset).fetch(&self.0);

        // while let Some(Ok(e)) = query.next().await {
        //     vec_resp.push(T::from(e));
        // }

        tracing::debug!("sql query result - {:?}", vec_resp);

        if vec_resp.is_empty() {
            Err(RepositoryError::RowNotFound)
        } else {
            Ok(vec_resp)
        }
    }

    async fn update<T, U>(
        &self,
        updater: U,
        condition: impl MapQuery,
    ) -> ResultRepository<QueryResult<T>>
    where
        T: Table + Debug,
        U: Updatable + Debug,
    {
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

    async fn delete<T>(&self, condition: impl MapQuery) -> ResultRepository<QueryResult<T>>
    where
        T: Table + Debug,
    {
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
