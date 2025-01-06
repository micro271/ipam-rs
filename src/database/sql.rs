use super::repository::{Table, TypeTable};
use sqlx::{postgres::PgArguments, query::Query, Postgres};
use std::collections::HashMap;

pub struct SqlOperations;

impl SqlOperations {
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
                TypeTable::OptionVlan(value) => sql.bind(value),
                TypeTable::I64(value) => sql.bind(value),
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
                    TypeTable::OptionVlan(e) => sql.bind(e),
                    TypeTable::OptionUuid(e) => sql.bind(e),
                    TypeTable::String(s) => sql.bind(s),
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
