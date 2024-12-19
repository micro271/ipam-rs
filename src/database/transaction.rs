use super::{repository::error::RepositoryError, Table, TypeTable, Updatable};
use sqlx::{Postgres, Transaction as SqlxTransaction};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Mutex;

pub trait Transaction {
    fn execute() -> impl Future<Output = Result<(), RepositoryError>>;
}

type ResultTransaction<'a> = Pin<Box<dyn Future<Output = Result<(), RepositoryError>> + 'a + Send>>;

pub struct BuilderPgTransaction<'a> {
    transaction: Arc<Mutex<SqlxTransaction<'a, Postgres>>>,
    to_update: Option<ResultTransaction<'a>>,
    to_insert: Option<ResultTransaction<'a>>,
    to_delete: Option<ResultTransaction<'a>>,
    _state: FutureState,
}

#[derive(Debug, Clone)]
enum FutureState {
    State0,
    Update,
    Insert,
    Delete,
    Success,
}

impl<'b> BuilderPgTransaction<'b> {
    pub async fn new(transaction: Arc<Mutex<SqlxTransaction<'b, Postgres>>>) -> Self {
        Self {
            transaction,
            to_update: None,
            to_insert: None,
            to_delete: None,
            _state: FutureState::State0,
        }
    }

    pub async fn insert<T>(mut self, data: Vec<T>) -> Self
    where
        T: Table + Send + std::fmt::Debug + Clone + 'static,
    {
        let transaction = self.transaction.clone();

        self.to_insert = Some(
            Box::pin(async move {
                let mut transaction = transaction.lock().await;
                let q_insert = T::query_insert();
                for i in data {
                    let mut sql = sqlx::query(&q_insert);
                    let fields = i.get_fields();
                    for element in fields {
                        sql = match element {
                            TypeTable::String(value) => sql.bind(value),
                            TypeTable::OptionUuid(value) => sql.bind(value),
                            TypeTable::Uuid(value) => sql.bind(value),
                            TypeTable::OptionString(value) => sql.bind(value),
                            TypeTable::Status(value) => sql.bind(value),
                            TypeTable::Role(value) => sql.bind(value),
                            TypeTable::OptionVlan(value) => sql.bind(value),
                            TypeTable::OptionCredential(value) => sql.bind(value),
                            TypeTable::I64(value) => sql.bind(value),
                            TypeTable::Null => sql,
                        };
                    }
                    let _ = sql.execute(&mut **transaction).await;
                }
                Ok(())
            }),
        );
        self
    }

    pub fn update<T, U>(
        mut self,
        updater: U,
        condition: Option<HashMap<&'static str, TypeTable>>,
    ) -> Self
    where
        T: Table + std::fmt::Debug + Clone,
        U: Updatable<'b> + Send + std::fmt::Debug,
    {
        self
    }

    pub fn delete<'a, T>(
        &'a self,
        condition: Option<HashMap<&'a str, TypeTable>>,
    ) -> super::ResultRepository<'a, super::QueryResult<T>>
    where
        T: Table + 'a + Send + std::fmt::Debug + Clone,
    {
        todo!()
    }
}

impl<'b> std::future::Future for BuilderPgTransaction<'b> {
    type Output = Result<(), RepositoryError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        match this._state {
            FutureState::State0 => {
                this._state = FutureState::Insert;
                Poll::Pending
            }
            FutureState::Insert => {
                if let Some(future) = this.to_insert.as_mut() {
                    if let Poll::Ready(resp) = future.as_mut().poll(cx) {
                        match resp {
                            Ok(_) => this._state = FutureState::Update,
                            Err(e) => return Poll::Ready(Err(e)),
                        }
                    }
                } else {
                    this._state = FutureState::Update
                }
                Poll::Pending
            }
            FutureState::Update => {
                if let Some(future) = this.to_update.as_mut() {
                    if let Poll::Ready(resp) = future.as_mut().poll(cx) {
                        match resp {
                            Ok(_) => this._state = FutureState::Delete,
                            Err(e) => return Poll::Ready(Err(e)),
                        }
                    }
                } else {
                    this._state = FutureState::Delete;
                }
                Poll::Pending
            }
            FutureState::Delete => {
                if let Some(future) = this.to_delete.as_mut() {
                    if let Poll::Ready(resp) = future.as_mut().poll(cx) {
                        match resp {
                            Ok(_) => {
                                this._state = FutureState::Success;
                            }
                            Err(e) => return Poll::Ready(Err(e)),
                        }

                        Poll::Pending
                    } else {
                        Poll::Pending
                    }
                } else {
                    this._state = FutureState::Success;
                    Poll::Pending
                }
            }
            FutureState::Success => Poll::Ready(Ok(())),
        }
    }
}
