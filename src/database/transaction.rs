use super::{
    repository::{error::RepositoryError, Repository}, sql::SqlOperations, Table, TypeTable, Updatable
};
use sqlx::{Executor, Postgres, Transaction as SqlxTransaction};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Mutex;

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
        U: Updatable<'b> + Send + std::fmt::Debug + 'static,
    {
        let transaction = self.transaction.clone();

        TransactionTask::new(async move {
            if let Some(pair) = updater.get_pair() {
                let mut transaction = transaction.lock().await;
                let cols = T::columns();

                let mut query = format!("UPDATE {} SET", T::name());

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
                        TypeTable::OptionCredential(e) => sql.bind(e),
                        TypeTable::OptionVlan(e) => sql.bind(e),
                        TypeTable::String(s) => sql.bind(s),
                        TypeTable::OptionString(value) => sql.bind(value),
                        TypeTable::Status(value) => sql.bind(value),
                        TypeTable::Uuid(e) => sql.bind(e),
                        TypeTable::Role(value) => sql.bind(value),
                        TypeTable::OptionUuid(e) => sql.bind(e),
                        TypeTable::Null => sql,
                        TypeTable::I64(e) => sql.bind(e),
                    };
                }
                let _ = sql.execute(&mut **transaction).await;

                Ok(())
            } else {
                Err(RepositoryError::ColumnNotFound("".to_string()))
            }
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
            let mut query = format!("DELETE FROM {}", T::name());
            let mut transaction = transaction.lock().await;
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
                            TypeTable::OptionCredential(e) => ex.bind(e),
                            TypeTable::OptionVlan(e) => ex.bind(e),
                            TypeTable::OptionUuid(e) => ex.bind(e),
                            TypeTable::String(s) => ex.bind(s),
                            TypeTable::OptionString(s) => ex.bind(s),
                            TypeTable::Uuid(e) => ex.bind(e),
                            TypeTable::Status(status) => ex.bind(status),
                            TypeTable::Role(role) => ex.bind(role),
                            TypeTable::I64(e) => ex.bind(e),
                            TypeTable::Null => ex,
                        };
                    }

                    let _ = ex.execute(&mut **transaction).await;
                    Ok(())
                }

                None => {
                    let _ = sqlx::query(&query).execute(&mut **transaction).await;
                    Ok(())
                }
                _ => Err(RepositoryError::ColumnNotFound("".to_string())),
            }
        })
    }
}

pub struct TransactionTask<'a> {
    inner: Pin<Box<dyn Future<Output = Result<(), RepositoryError>> + 'a + Send>>,
}

impl<'a> TransactionTask<'a> {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = Result<(), RepositoryError>> + 'a + Send,
    {
        Self {
            inner: Box::pin(future),
        }
    }
}

impl<'a> Future for TransactionTask<'a> {
    type Output = Result<(), RepositoryError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.inner.as_mut().poll(cx)
    }
}