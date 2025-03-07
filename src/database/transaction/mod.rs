pub mod error;
use crate::handler::MapQuery;

use super::{
    Table, Updatable,
    repository::{QueryResult, Repository, error::RepositoryError},
    sql::SqlOperations,
};
use sqlx::{Postgres, Transaction as SqlxTransaction};
use std::sync::Arc;

use tokio::sync::Mutex;

type TransactionResult<T> = Result<QueryResult<T>, RepositoryError>;

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

    pub async fn insert<T: Table>(&mut self, data: T) -> TransactionResult<T> {
        let mut transaction = self.transaction.lock().await;
        let q_insert = T::query_insert();
        let query = SqlOperations::insert(data, &q_insert);

        Ok(QueryResult::Insert(
            query.execute(&mut **transaction).await?.rows_affected(),
        ))
    }

    pub async fn update<T: Table, U: Updatable, M: MapQuery>(
        &mut self,
        updater: U,
        condition: M,
    ) -> TransactionResult<T> {
        let mut query = T::query_update();
        let sql = SqlOperations::update(
            updater.get_pair().unwrap(),
            condition.get_pairs(),
            &mut query,
        );

        let mut transaction = self.transaction.lock().await;
        let resp = sql.execute(&mut **transaction).await?;
        Ok(QueryResult::Update(resp.rows_affected()))
    }

    pub async fn delete<T: Table, M: MapQuery>(&mut self, condition: M) -> TransactionResult<T> {
        let mut query = T::query_delete();
        let sql = SqlOperations::delete(condition.get_pairs(), &mut query);
        let mut transaction = self.transaction.lock().await;
        let resp = sql.execute(&mut **transaction).await?;

        Ok(QueryResult::Delete(resp.rows_affected()))
    }
}
