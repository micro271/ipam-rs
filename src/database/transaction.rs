use super::{repository::{error::RepositoryError, QueryResult}, Table, TypeTable, Updatable};
use sqlx::{Postgres, Transaction as SqlxTransaction};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Mutex;


type ResultTransaction<'a> = Pin<Box<dyn Future<Output = Result<(), RepositoryError>> + 'a + Send>>;

pub struct BuilderPgTransaction<'a> {
    transaction: Arc<Mutex<SqlxTransaction<'a, Postgres>>>,
    to_update: Vec<ResultTransaction<'a>>,
    to_insert: Vec<ResultTransaction<'a>>,
    to_delete: Vec<ResultTransaction<'a>>,
    _state: FutureState,
    pos: usize,
}

#[derive(Debug, Clone)]
enum FutureState {
    Update,
    Insert,
    Delete,
}

impl<'b> BuilderPgTransaction<'b> {
    pub async fn new(transaction: Arc<Mutex<SqlxTransaction<'b, Postgres>>>) -> Self {
        Self {
            transaction,
            to_update: Vec::new(),
            to_insert: Vec::new(),
            to_delete: Vec::new(),
            _state: FutureState::Insert,
            pos: 0,
        }
    }

    pub fn insert<T>(&mut self, data: Vec<T>)
    where
        T: Table + Send + std::fmt::Debug + Clone + 'b,
    {
        let transaction = self.transaction.clone();

        self.to_insert.push(
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
    }

    pub fn update<T, U>(
        &mut self,
        updater: U,
        condition: Option<HashMap<&'static str, TypeTable>>,
    )
    where
        T: Table + std::fmt::Debug + Clone,
        U: Updatable<'b> + Send + std::fmt::Debug + 'static,
    {
        let transaction = self.transaction.clone();

        self.to_update.push(Box::pin(async move {

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
        }));
    }

    pub fn delete<T>(
        &mut self,
        condition: Option<HashMap<&'static str, TypeTable>>,
    )
    where
        T: Table + 'b + Send + std::fmt::Debug + Clone,
    {

        let transaction = self.transaction.clone();
        self.to_delete.push(Box::pin(async move {
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
                },
                _ => Err(RepositoryError::ColumnNotFound("".to_string())),
            }
        }));
    }
}

impl<'b> std::future::Future for BuilderPgTransaction<'b> {
    type Output = Result<(), RepositoryError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            println!("Ingresamos al loop");
            match this._state {
                FutureState::Insert => {
                    if let Some(future) = this.to_insert.get_mut(this.pos) {
                        if let Poll::Ready(resp) = future.as_mut().poll(cx) {
                            match resp {
                                Ok(_) => this.pos += 1,
                                Err(e) => return Poll::Ready(Err(e)),
                            }
                        } else {
                            return Poll::Pending
                        }
                    } else {
                        this.pos = 0;
                        this._state = FutureState::Update;
                    }
                }
                FutureState::Update => {
                    if let Some(future) = this.to_update.get_mut(this.pos) {
                        if let Poll::Ready(resp) = future.as_mut().poll(cx) {
                            match resp {
                                Ok(_) => this.pos += 1,
                                Err(e) => return Poll::Ready(Err(e)),
                            }
                        } else {
                            return Poll::Pending;
                        }
                    } else {
                        this.pos = 0;
                        this._state = FutureState::Delete;
                    }
                }
                FutureState::Delete => {
                    if let Some(future) = this.to_delete.get_mut(this.pos) {
                        return if let Poll::Ready(resp) = future.as_mut().poll(cx) {
                            Poll::Ready(resp)
                        } else {
                            Poll::Pending
                        };
                    } else {
                        return Poll::Ready(Ok(()))
                    }
                }
            }
        }
    }
}
