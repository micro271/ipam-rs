pub mod error;

use super::{
    Table, Updatable,
    repository::{MapQuery, QueryResult, Repository, error::RepositoryError},
    sql::SqlOperations,
};
use futures::StreamExt;
use sqlx::{Postgres, Transaction as SqlxTransaction, postgres::PgRow};
use std::sync::Arc;

use tokio::sync::Mutex;

type TransactionResult<T> = Result<T, RepositoryError>;

pub trait Transaction: Repository {
    fn transaction(
        &self,
    ) -> impl Future<Output = Result<BuilderPgTransaction<'_>, RepositoryError>>;
}

pub struct BuilderPgTransaction<'a> {
    pub(super) transaction: Arc<Mutex<SqlxTransaction<'a, Postgres>>>,
}

impl<'b> BuilderPgTransaction<'b> {
    pub fn new(transaction: SqlxTransaction<'b, Postgres>) -> Self {
        Self {
            transaction: Arc::new(Mutex::new(transaction)),
        }
    }

    pub async fn commit(self) -> Result<(), RepositoryError> {
        // TODO: We have to create an error that informs that there are some transactions without finishing
        let tmp = self;

        let transaction = Arc::try_unwrap(tmp.transaction).unwrap().into_inner();
        transaction.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), RepositoryError> {
        // TODO: We have to create an error that informs that there are some transactions without finishing
        let tmp = self;

        let transaction = Arc::try_unwrap(tmp.transaction).unwrap().into_inner();
        transaction.rollback().await?;
        Ok(())
    }

    pub async fn get<T: Table + From<PgRow>>(
        &mut self,
        condition: impl MapQuery,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> TransactionResult<Vec<T>> {
        let mut transaction = self.transaction.lock().await;
        let mut query = T::query_select();
        let query = SqlOperations::get(&mut query, condition, limit, offset);
        let mut cursor = query.fetch(&mut **transaction);
        let mut resp = Vec::new();
        while let Some(Ok(e)) = cursor.next().await {
            resp.push(e.into());
        }

        Ok(resp)
    }

    pub async fn insert<T: Table>(&mut self, data: T) -> TransactionResult<QueryResult> {
        let mut transaction = self.transaction.lock().await;
        let q_insert = T::query_insert();
        let query = SqlOperations::insert(data, &q_insert);

        Ok(query.execute(&mut **transaction).await?.into())
    }

    pub async fn update<T: Table, U: Updatable, M: MapQuery>(
        &mut self,
        updater: U,
        condition: M,
    ) -> TransactionResult<QueryResult> {
        let mut query = T::query_update();
        let sql = SqlOperations::update(
            updater.get_pair().unwrap(),
            condition.get_pairs(),
            &mut query,
        );

        let mut transaction = self.transaction.lock().await;

        Ok(sql.execute(&mut **transaction).await?.into())
    }

    pub async fn delete<T: Table, M: MapQuery>(
        &mut self,
        condition: M,
    ) -> TransactionResult<QueryResult> {
        let mut query = T::query_delete();
        let sql = SqlOperations::delete(condition.get_pairs(), &mut query);
        let mut transaction = self.transaction.lock().await;

        Ok(sql.execute(&mut **transaction).await?.into())
    }
}
