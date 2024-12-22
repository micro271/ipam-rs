use super::{
    repository::{error::RepositoryError, Repository},
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

type TransactionTaskResult = Result<(), RepositoryError>;

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

    pub fn insert<T>(&mut self, data: T) -> TransactionTask<'_>
    where
        T: Table + Send + std::fmt::Debug + Clone + 'b,
    {
        let transaction = self.transaction.clone();
        TransactionTask::new(async move {
            let mut transaction = transaction.lock().await;
            let q_insert = T::query_insert();
            let query = SqlOperations::insert(data, &q_insert);
            let _ = query.execute(&mut **transaction).await;
            Ok(())
        })
    }

    pub fn update<T, U>(
        &mut self,
        updater: U,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> TransactionTask<'_>
    where
        T: Table + std::fmt::Debug + Clone,
        U: Updatable<'static> + Send + std::fmt::Debug + 'static,
    {
        let transaction = self.transaction.clone();

        TransactionTask::new(async move {
            let mut query = T::query_update();
            let sql = SqlOperations::update(updater.get_pair().unwrap(), condition, &mut query);

            let mut transaction = transaction.lock().await;
            let _ = sql.execute(&mut **transaction).await;
            Ok(())
        })
    }

    pub fn delete<T>(
        &mut self,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> TransactionTask<'_>
    where
        T: Table + 'b + Send + std::fmt::Debug + Clone,
    {
        let transaction = self.transaction.clone();
        TransactionTask::new(async move {
            let mut query = T::query_delete();
            let sql = SqlOperations::delete(condition, &mut query);
            let mut transaction = transaction.lock().await;
            sql.execute(&mut **transaction).await?;
            Ok(())
        })
    }
}

pub struct TransactionTask<'a> {
    inner: Pin<Box<dyn Future<Output = TransactionTaskResult> + 'a + Send>>,
}

impl<'a> TransactionTask<'a> {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = TransactionTaskResult> + 'a + Send,
    {
        Self {
            inner: Box::pin(future),
        }
    }
}

impl<'a> Future for TransactionTask<'a> {
    type Output = TransactionTaskResult;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.inner.as_mut().poll(cx)
    }
}
