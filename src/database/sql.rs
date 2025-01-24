use super::repository::{Table, TypeTable};
use sqlx::{postgres::PgArguments, query::Query, Postgres};
use std::collections::HashMap;
use crate::MapQuery;

pub struct SqlOperations;
// We've taken ownership of the query string and used a builder to add the behavior of limit, offset, and group by.

impl SqlOperations {

    pub fn get<'a>(query: &'a mut String, condition: impl MapQuery, limit: Option<i32>, offset: Option<i32>) -> Query<'a, Postgres, PgArguments> { 
        let condition = condition.get_pairs();

        if let Some(col) = condition {
            query.push_str(" WHERE");

            let mut data_pos = HashMap::new();

            let mut pos = 1;
            let len = col.len();

            for (key, value) in col {
                if value == TypeTable::Null {
                    query.push_str(&format!(" {} IS NULL", key));
                } else {
                    query.push_str(&format!(" {} = ${}", key, pos));
                    if pos < len {
                        query.push_str(" AND");
                    }
                    data_pos.insert(pos, value);
                    pos += 1;
                }
            }

            if let Some(limit) = limit {
                query.push_str(&format!(" LIMIT {}", limit));
            }
            if let Some(offset) = offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
            let mut resp = sqlx::query(query);

            for i in 1..pos {
                resp = match data_pos.remove(&i).unwrap() {
                    TypeTable::OptionUuid(e) => resp.bind(e),
                    TypeTable::Uuid(e) => resp.bind(e),
                    TypeTable::String(s) => resp.bind(s),
                    TypeTable::OptionString(opt) => resp.bind(opt),
                    TypeTable::Status(status) => resp.bind(status),
                    TypeTable::Role(role) => resp.bind(role),
                    TypeTable::OptionVlanId(e) => resp.bind(e),
                    TypeTable::Bool(e) => resp.bind(e),
                    TypeTable::OptionTime(e) => resp.bind(e),
                    TypeTable::Time(e) => resp.bind(e),
                    TypeTable::VlanId(e) => resp.bind(e),
                    TypeTable::I64(e) => resp.bind(e),
                    TypeTable::I32(e) => resp.bind(e),
                    TypeTable::Null => resp,
                };
            }
            resp
        } else {
            sqlx::query(query)
        }
    }
    pub fn insert<T>(data: T, query: &str) -> Query<'_, Postgres, PgArguments>
    where
        T: Table + std::fmt::Debug + Clone,
    {
        let mut sql = sqlx::query(query);
        let fields = data.get_fields();
        for element in fields {
            sql = match element {
                TypeTable::String(value) => sql.bind(value),
                TypeTable::OptionUuid(value) => sql.bind(value),
                TypeTable::Uuid(value) => sql.bind(value),
                TypeTable::OptionString(value) => sql.bind(value),
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
        let len: usize = pair_updater.len();
        for (key, value) in pair_updater {
            if value == TypeTable::Null {
                query.push_str(&format!(" {} = NULL", key));
            } else {
                query.push_str(&format!(" {} = ${}", key, pos));
                pos_values.insert(pos, value);

                if len > pos {
                    query.push(',');
                }
                pos += 1
            }
        }

        if let Some(condition) = condition {
            let len = condition.len() + pos - 1;
            for (key, value) in condition {
                if value == TypeTable::Null {
                    query.push_str(&format!(" {} IS NULL", key));
                } else {
                    pos_values.insert(pos, value);
                    query.push_str(&format!(" {} = ${}", key, pos));

                    if pos < len {
                        query.push_str(" AND");
                    }
                    pos += 1;
                }
            }
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
        if let Some(condition) = condition {
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
            sqlx::query(query)
        }
    }
}
