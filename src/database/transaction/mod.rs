pub mod error;
use super::{
    repository::{error::RepositoryError, QueryResult, Repository},
    sql::SqlOperations,
    Table, TypeTable, Updatable,
};
use sqlx::{Postgres, Transaction as SqlxTransaction};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Mutex;

type TransactionTaskResult<T> = Result<QueryResult<T>, RepositoryError>;

pub trait Transaction<'a>: Repository {
    fn transaction(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<BuilderPgTransaction<'a>, RepositoryError>> + 'a + Send>>;
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

    pub fn insert<T>(
        &mut self,
        data: T,
    ) -> impl Future<Output = TransactionTaskResult<T>> + use<'_, T>
    where
        T: Table + Send + std::fmt::Debug + Clone + 'b,
    {
        let transaction = self.transaction.clone();
        TransactionTask::new(async move {
            let mut transaction = transaction.lock().await;
            let q_insert = T::query_insert();
            let query = SqlOperations::insert(data, &q_insert);

            Ok(QueryResult::Insert(
                query.execute(&mut **transaction).await?.rows_affected(),
            ))
        })
    }

    pub fn update<T, U>(
        &mut self,
        updater: U,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> impl Future<Output = TransactionTaskResult<T>> + use<'_, T, U>
    where
        T: Table + std::fmt::Debug + Clone,
        U: Updatable + Send + std::fmt::Debug + 'static,
    {
        let transaction = self.transaction.clone();

        TransactionTask::new(async move {
            let mut query = T::query_update();
            let sql = SqlOperations::update(updater.get_pair().unwrap(), condition, &mut query);

            let mut transaction = transaction.lock().await;
            Ok(QueryResult::Update(
                sql.execute(&mut **transaction).await?.rows_affected(),
            ))
        })
    }

    pub fn delete<T>(
        &mut self,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> impl Future<Output = TransactionTaskResult<T>> + use<'_, T>
    where
        T: Table + 'b + Send + std::fmt::Debug + Clone,
    {
        let transaction = self.transaction.clone();
        TransactionTask::new(async move {
            let mut query = T::query_delete();
            let sql = SqlOperations::delete(condition, &mut query);
            let mut transaction = transaction.lock().await;
            Ok(QueryResult::Delete(
                sql.execute(&mut **transaction).await?.rows_affected(),
            ))
        })
    }
}

pub struct TransactionTask<'a, T> {
    future: Pin<Box<dyn Future<Output = TransactionTaskResult<T>> + 'a + Send>>,
}

impl<'a, T> TransactionTask<'a, T> {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = TransactionTaskResult<T>> + 'a + Send,
    {
        Self {
            future: Box::pin(future),
        }
    }
}

impl<T> Future for TransactionTask<'_, T> {
    type Output = TransactionTaskResult<T>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.future.as_mut().poll(cx)
    }
}
