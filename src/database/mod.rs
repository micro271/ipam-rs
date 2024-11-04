pub mod mappers;
pub mod repository;
pub mod utils;

use crate::models::utils::*;
use futures::stream::StreamExt;
use repository::{error::RepositoryError, QueryResult, Repository, ResultRepository};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use std::collections::HashMap;

pub struct PgRepository(PgPool);

impl PgRepository {
    pub async fn new(url: String) -> Result<Self, RepositoryError> {
        Ok(Self(
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await?,
        ))
    }
}

impl Repository for PgRepository {
    fn insert<'a, T>(&'a self, data: Vec<T>) -> ResultRepository<'a, QueryResult>
    where
        T: Table + 'a + Send,
    {
        let resp = async {
            let mut tx = match self.begin().await {
                Ok(e) => e,
                Err(e) => return Err(RepositoryError::Sqlx(e.to_string())),
            };

            let mut count = 0;
            for data in data {
                let query = T::query_insert();
                let mut tmp = sqlx::query(&query);
                let data = T::get_fields(data);
                println!("{:?}", data);
                for i in data {
                    tmp = match i {
                        TypeTable::String(s) => tmp.bind(s),
                        TypeTable::OptionCredential(e) => tmp.bind(e),
                        TypeTable::OptionString(opt) => tmp.bind(opt),
                        TypeTable::OptionVlan(e) => tmp.bind(e),
                        TypeTable::Status(status) => tmp.bind(status),
                        TypeTable::Int32(i) => tmp.bind(i),
                        TypeTable::Uuid(e) => tmp.bind(e),
                        TypeTable::Role(r) => tmp.bind(r),
                        TypeTable::Float64(f) => tmp.bind(f),
                        TypeTable::OptionUuid(e) => tmp.bind(e),
                    };
                }

                match tmp.execute(&mut *tx).await {
                    Ok(_) => {
                        count += 1;
                    }
                    Err(e) => {
                        tx.rollback().await?;
                        return Err(RepositoryError::Sqlx(e.to_string()));
                    }
                }
            }

            match tx.commit().await {
                Ok(_) => Ok(QueryResult::Insert(count)),
                Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
            }
        };
        Box::pin(resp)
    }

    fn get<'a, T>(
        &'a self,
        column_data: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, Vec<T>>
    where
        T: Table + From<PgRow> + 'a + Send,
    {
        Box::pin(async {
            let mut query = format!("SELECT * FROM {}", T::name());
            let mut vec_resp = Vec::new();

            match column_data {
                Some(col) if !col.is_empty() => {
                    let cols = T::columns();
                    query.push_str(" WHERE");

                    let mut data_pos = HashMap::new();

                    let mut pos = 1;
                    let len = col.keys().len();
                    for i in col.keys() {
                        if !cols.contains(i) {
                            return Err(RepositoryError::ColumnNotFound(Some(i.to_string())));
                        }

                        query.push_str(&format!(" {} = ${}", i, pos));
                        if len > pos {
                            query.push_str(" AND");
                        }
                        data_pos.insert(pos, col.get(i).unwrap());
                        pos += 1;
                    }

                    let mut resp = sqlx::query(&query);

                    for i in 1..pos {
                        resp = match data_pos.get(&i).unwrap() {
                            TypeTable::OptionUuid(e) => resp.bind(e),
                            TypeTable::OptionCredential(e) => resp.bind(e),
                            TypeTable::OptionVlan(e) => resp.bind(e),
                            TypeTable::Uuid(e) => resp.bind(e),
                            TypeTable::String(s) => resp.bind(s),
                            TypeTable::OptionString(opt) => resp.bind(opt),
                            TypeTable::Status(status) => resp.bind(status),
                            TypeTable::Int32(num) => resp.bind(num),
                            TypeTable::Role(role) => resp.bind(role),
                            TypeTable::Float64(f) => resp.bind(f),
                        };
                    }

                    let mut resp = resp.fetch(&**self);
                    while let Some(Ok(device)) = resp.next().await {
                        vec_resp.push(T::from(device));
                    }

                    if !vec_resp.is_empty() {
                        Ok(vec_resp)
                    } else {
                        Err(RepositoryError::RowNotFound)
                    }
                }
                None => Ok({

                    let mut aux = sqlx::query(&query).fetch(&**self);
                    while let Some(Ok(e)) = aux.next().await {
                        vec_resp.push(T::from(e));
                    }

                    vec_resp
                }),
                _ => Err(RepositoryError::ColumnNotFound(None)),
            }
        })
    }

    fn update<'a, T, U>(
        &'a self,
        updater: U,
        condition: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, QueryResult>
    where
        T: Table + 'a + Send,
        U: Updatable<'a> + 'a + Send,
    {
        let tmp = async move {
            if let Some(pair) = updater.get_pair() {
                let cols = T::columns();

                let mut query = format!("UPDATE {} SET", T::name());

                let mut pos_values = HashMap::new();

                let mut pos = 1;
                for i in pair.keys() {
                    if !cols.contains(i) {
                        return Err(RepositoryError::ColumnNotFound(Some(i.to_string())));
                    }

                    query.push_str(&format!(" {} = ${}", i, pos));

                    pos_values.insert(pos, pair.get(i).unwrap());
                    pos += 1;
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

                    if len > pos {
                        query.push_str(" AND");
                    }
                    pos += 1;
                }

                let mut sql = sqlx::query(&query);
                for i in 1..pos {
                    sql = match pos_values.get(&i).unwrap() {
                        TypeTable::String(s) => sql.bind(s),
                        TypeTable::OptionString(value) => sql.bind(value),
                        TypeTable::Status(value) => sql.bind(value),
                        TypeTable::Int32(value) => sql.bind(value),
                        TypeTable::Uuid(e) => sql.bind(e),
                        TypeTable::Role(value) => sql.bind(value),
                        TypeTable::Float64(value) => sql.bind(value),
                        TypeTable::OptionUuid(e) => sql.bind(e),
                        TypeTable::OptionVlan(e) => sql.bind(e),
                        TypeTable::OptionCredential(e) => sql.bind(e),
                    };
                }

                match sql.execute(&**self).await {
                    Ok(e) => Ok(QueryResult::Update(e.rows_affected())),
                    Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
                }
            } else {
                Err(RepositoryError::ColumnNotFound(None))
            }
        };
        Box::pin(tmp)
    }

    fn delete<'a, T>(
        &'a self,
        condition: Option<HashMap<&'a str, TypeTable>>,
    ) -> ResultRepository<'a, QueryResult>
    where
        T: Table + 'a + Send,
    {
        let resp = async move {
            let mut query = format!("DELETE FROM {}", T::name());

            match condition {
                Some(condition) if !condition.is_empty() => {
                    let columns = T::columns();

                    query.push_str(" WHERE");

                    let mut pos_column = HashMap::new();
                    let mut pos = 1;

                    for t in condition.keys() {
                        if !columns.contains(t) {
                            return Err(RepositoryError::ColumnNotFound(Some(t.to_string())));
                        }

                        query.push_str(&format!(" {} = ${}", t, pos));
                        pos_column.insert(pos, condition.get(t).unwrap());
                        pos += 1;
                    }

                    let mut ex = sqlx::query(&query);

                    for i in 1..pos {
                        ex = match pos_column.get(&i).unwrap() {
                            TypeTable::OptionUuid(e) => ex.bind(e),
                            TypeTable::String(s) => ex.bind(s),
                            TypeTable::OptionString(s) => ex.bind(s),
                            TypeTable::Uuid(e) => ex.bind(e),
                            TypeTable::Status(status) => ex.bind(status),
                            TypeTable::Int32(i) => ex.bind(i),
                            TypeTable::Role(role) => ex.bind(role),
                            TypeTable::Float64(f) => ex.bind(f),
                            TypeTable::OptionVlan(e) => ex.bind(e),
                            TypeTable::OptionCredential(e) => ex.bind(e),
                        };
                    }

                    match ex.execute(&**self).await {
                        Ok(e) => Ok(QueryResult::Delete(e.rows_affected())),
                        Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
                    }
                }

                None => match sqlx::query(&query).execute(&**self).await {
                    Ok(e) => Ok(QueryResult::Delete(e.rows_affected())),
                    Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
                },
                _ => Err(RepositoryError::ColumnNotFound(None)),
            }
        };

        Box::pin(resp)
    }
}

impl std::ops::Deref for PgRepository {
    type Target = PgPool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PgRepository {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}