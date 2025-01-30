use super::repository::{Table, TypeTable};
use crate::MapQuery;
use sqlx::{postgres::PgArguments, query::Query, Postgres};
use std::collections::HashMap;

pub struct SqlOperations;
// We've taken ownership of the query string and used a builder to add the behavior of limit, offset, and group by.

impl SqlOperations {
    pub fn get(
        query: &mut String,
        condition: impl MapQuery,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Query<'_, Postgres, PgArguments> {
        tracing::trace!("SQL OPERATIONS");
        tracing::trace!("1 input (query) - {}", query);
        tracing::trace!("2 input (condition) - {:?}", condition);
        tracing::trace!("3 input (limit) - {:?}", limit);
        tracing::trace!("4 input (offset) - {:?}", offset);

        let condition = condition.get_pairs();

        if let Some(col) = condition {
            tracing::trace!("5 if (condition exists) - {:?}", col);

            query.push_str(" WHERE");

            let mut data_pos = HashMap::new();

            let mut pos = 1;
            let len = col.len() - 1;

            for (i, (key, value)) in col.into_iter().enumerate() {
                if value == TypeTable::Null {
                    query.push_str(&format!(" {} IS NULL", key));
                } else {
                    query.push_str(&format!(" {} = ${}", key, pos));
                    data_pos.insert(pos, value);
                    pos += 1;
                }

                if i < len {
                    query.push_str(" AND");
                }
            }

            tracing::trace!("6 update (query) - {}", query);

            if let Some(limit) = limit {
                query.push_str(&format!(" LIMIT {}", limit));
                tracing::trace!("6 update (query) - {}", query);
            }
            if let Some(offset) = offset {
                query.push_str(&format!(" OFFSET {}", offset));
                tracing::trace!("6 update (query) - {}", query);
            }
            let mut sql = sqlx::query(query);

            for i in 1..pos {
                sql = match data_pos.remove(&i).unwrap() {
                    TypeTable::OptionUuid(e) => sql.bind(e),
                    TypeTable::To(e) => sql.bind(e),
                    TypeTable::Uuid(e) => sql.bind(e),
                    TypeTable::String(s) => sql.bind(s),
                    TypeTable::OptionString(opt) => sql.bind(opt),
                    TypeTable::Status(status) => sql.bind(status),
                    TypeTable::Role(role) => sql.bind(role),
                    TypeTable::OptionVlanId(e) => sql.bind(e),
                    TypeTable::Bool(e) => sql.bind(e),
                    TypeTable::OptionTime(e) => sql.bind(e),
                    TypeTable::Time(e) => sql.bind(e),
                    TypeTable::VlanId(e) => sql.bind(e),
                    TypeTable::I64(e) => sql.bind(e),
                    TypeTable::I32(e) => sql.bind(e),
                    TypeTable::Null => sql,
                };
            }
            sql
        } else {
            sqlx::query(query)
        }
    }
    pub fn insert<T>(data: T, query: &str) -> Query<'_, Postgres, PgArguments>
    where
        T: Table + std::fmt::Debug + Clone,
    {
        tracing::trace!("1 input (data) - {:?}", data);
        tracing::trace!("2 input (query) - {}", query);

        let mut sql = sqlx::query(query);
        let fields = data.get_fields();
        for element in fields {
            sql = match element {
                TypeTable::String(value) => sql.bind(value),
                TypeTable::OptionUuid(value) => sql.bind(value),
                TypeTable::Uuid(value) => sql.bind(value),
                TypeTable::OptionString(value) => sql.bind(value),
                TypeTable::To(e) => sql.bind(e),
                TypeTable::Status(value) => sql.bind(value),
                TypeTable::Role(value) => sql.bind(value),
                TypeTable::I32(e) => sql.bind(e),
                TypeTable::OptionVlanId(value) => sql.bind(value),
                TypeTable::VlanId(e) => sql.bind(e),
                TypeTable::I64(value) => sql.bind(value),
                TypeTable::Bool(e) => sql.bind(e),
                TypeTable::OptionTime(e) => sql.bind(e),
                TypeTable::Time(e) => sql.bind(e),
                TypeTable::Null => sql,
            };
        }

        sql
    }

    pub fn update<'a>(
        pair_updater: HashMap<&'_ str, TypeTable>,
        condition: Option<HashMap<&'_ str, TypeTable>>,
        query: &'a mut String,
    ) -> Query<'a, Postgres, PgArguments> {
        let mut pos_values = HashMap::new();

        let mut pos = 1;
        let len = pair_updater.len() - 1;

        tracing::trace!("1 input (value to update) - {:?}", pair_updater);
        tracing::trace!("2 input (condition) - {:?}", condition);
        tracing::trace!("3 input (query) - {}", query);

        for (i, (key, value)) in pair_updater.into_iter().enumerate() {
            if value == TypeTable::Null {
                query.push_str(&format!(" {} IS NULL", key));
            } else {
                query.push_str(&format!(" {} = ${}", key, pos));
                pos_values.insert(pos, value);
                pos += 1;
            }

            if len > i {
                query.push(',');
            }
        }

        tracing::trace!("2 (query update) - {}", query);

        if let Some(condition) = condition {
            let len = condition.len() - 1;
            tracing::trace!("3 if (condition exists) - {:?}", condition);

            query.push_str(" WHERE");

            for (i, (key, value)) in condition.into_iter().enumerate() {
                if value == TypeTable::Null {
                    query.push_str(&format!(" {} IS NULL", key));
                } else {
                    query.push_str(&format!(" {} = ${}", key, pos));
                    pos_values.insert(pos, value);
                    pos += 1;
                }

                if len > i {
                    query.push_str(" AND");
                }
            }

            tracing::trace!("4 update (query) - {}", query);
        }

        let mut sql = sqlx::query(query);
        for i in 1..pos {
            sql = match pos_values.remove(&i).unwrap() {
                TypeTable::OptionVlanId(e) => sql.bind(e),
                TypeTable::VlanId(e) => sql.bind(e),
                TypeTable::String(s) => sql.bind(s),
                TypeTable::OptionString(value) => sql.bind(value),
                TypeTable::Status(value) => sql.bind(value),
                TypeTable::Uuid(e) => sql.bind(e),
                TypeTable::I32(e) => sql.bind(e),
                TypeTable::To(e) => sql.bind(e),
                TypeTable::Role(value) => sql.bind(value),
                TypeTable::OptionUuid(e) => sql.bind(e),
                TypeTable::Bool(e) => sql.bind(e),
                TypeTable::OptionTime(e) => sql.bind(e),
                TypeTable::Time(e) => sql.bind(e),
                TypeTable::Null => sql,
                TypeTable::I64(e) => sql.bind(e),
            };
        }

        sql
    }

    pub fn delete<'a>(
        condition: Option<HashMap<&'_ str, TypeTable>>,
        query: &'a mut String,
    ) -> Query<'a, Postgres, PgArguments> {
        tracing::trace!("1 input (condition) - {:?}", condition);
        tracing::trace!("2 input (query) - {}", query);

        if let Some(condition) = condition {
            tracing::trace!("3 (condition exists) - {:?}", condition);

            query.push_str(" WHERE");

            let mut pos_column = HashMap::new();
            let mut pos = 1;

            let len = condition.len();
            for (key, value) in condition {
                if value == TypeTable::Null {
                    query.push_str(&format!(" {} IS NULL", key));
                } else {
                    query.push_str(&format!(" {} = ${}", key, pos));
                    pos_column.insert(pos, value);

                    if pos < len {
                        query.push_str(" AND");
                    }
                    pos += 1;
                }
            }

            tracing::trace!("4 (query update) - {:?}", query);

            let mut sql = sqlx::query(query);

            for i in 1..pos {
                sql = match pos_column.remove(&i).unwrap() {
                    TypeTable::OptionVlanId(e) => sql.bind(e),
                    TypeTable::VlanId(e) => sql.bind(e),
                    TypeTable::OptionUuid(e) => sql.bind(e),
                    TypeTable::Bool(e) => sql.bind(e),
                    TypeTable::OptionTime(e) => sql.bind(e),
                    TypeTable::Time(e) => sql.bind(e),
                    TypeTable::String(s) => sql.bind(s),
                    TypeTable::To(e) => sql.bind(e),
                    TypeTable::I32(e) => sql.bind(e),
                    TypeTable::OptionString(s) => sql.bind(s),
                    TypeTable::Uuid(e) => sql.bind(e),
                    TypeTable::Status(status) => sql.bind(status),
                    TypeTable::Role(role) => sql.bind(role),
                    TypeTable::I64(e) => sql.bind(e),
                    TypeTable::Null => sql,
                };
            }
            sql
        } else {
            tracing::trace!("3 condition not exists");
            sqlx::query(query)
        }
    }
}
