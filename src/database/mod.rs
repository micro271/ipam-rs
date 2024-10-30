pub mod utils;
use crate::models::utils::*;
use futures::stream::StreamExt;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use std::collections::HashMap;
use utils::*;

pub struct PgRepository(pub PgPool);

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
            let mut tx = match self.0.begin().await {
                Ok(e) => e,
                Err(e) => return Err(RepositoryError::Sqlx(e.to_string())),
            };

            let mut count = 0;
            for data in data {
                let query = T::query_insert();
                let mut tmp = sqlx::query(&query);
                let data = T::get_fields(data);

                for i in data {
                    tmp = match i {
                        TypeTable::String(s) => tmp.bind(s),
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
                    for i in col.keys() {
                        if !cols.contains(i) {
                            return Err(RepositoryError::ColumnNotFound(Some(i.to_string())));
                        }

                        query.push_str(&format!(" {} = ${}", i, pos));

                        data_pos.insert(pos, col.get(i).unwrap());
                        pos += 1;
                    }

                    let mut resp = sqlx::query(&query);

                    for i in 1..pos {
                        resp = match data_pos.get(&i).unwrap() {
                            TypeTable::OptionUuid(e) => resp.bind(e),
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

                    let mut resp = resp.fetch(&self.0);
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
                    vec_resp.push(T::from(sqlx::query(&query).fetch_one(&self.0).await?));
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
                    pos += 1
                }

                let condition = match condition {
                    Some(e) => {
                        query.push_str(" WHERE");
                        e
                    }
                    None => HashMap::new(),
                };

                for i in condition.keys() {
                    pos_values.insert(pos, condition.get(i).unwrap());
                    query.push_str(&format!(" {} = ${}", i, pos));
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
                    };
                }

                match sql.execute(&self.0).await {
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
                        };
                    }

                    match ex.execute(&self.0).await {
                        Ok(e) => Ok(QueryResult::Delete(e.rows_affected())),
                        Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
                    }
                }

                None => match sqlx::query(&query).execute(&self.0).await {
                    Ok(e) => Ok(QueryResult::Delete(e.rows_affected())),
                    Err(e) => Err(RepositoryError::Sqlx(e.to_string())),
                },
                _ => Err(RepositoryError::ColumnNotFound(None)),
            }
        };

        Box::pin(resp)
    }
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::Sqlx(txt) => write!(f, "Sqlx error: {}", txt),
            Self::RowNotFound => write!(f, "Row doesn't exist"),
            Self::ColumnNotFound(e) => match e {
                Some(e) => {
                    write!(f, "The column {} didn't find", e)
                }
                None => {
                    write!(f, "Undefined collumn")
                }
            },
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<sqlx::Error> for RepositoryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::RowNotFound,
            sqlx::Error::ColumnNotFound(e) => Self::ColumnNotFound(Some(e)),
            e => Self::Sqlx(e.to_string()),
        }
    }
}
