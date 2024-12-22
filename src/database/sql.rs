use sqlx::{postgres::PgArguments, Postgres, query::Query};
use std::collections::HashMap;
use super::repository::{Table, TypeTable, Updatable};


pub struct SqlOperations;

impl SqlOperations {
    pub fn insert<'a, T>(data: T, query: &'a str) -> Query<'a, Postgres, PgArguments>
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
                TypeTable::OptionCredential(value) => sql.bind(value),
                TypeTable::I64(value) => sql.bind(value),
                TypeTable::Null => sql,
            };
        }

        sql
    }
    
    pub fn update<'a, U>(
        pair_updater: &'a HashMap<&'a str, TypeTable>,
        condition: Option<&'a HashMap<&'a str, TypeTable>>,
        query: &'a mut String,
    ) -> Query<'a, Postgres, PgArguments>
    where
        U: Updatable<'a> + Send + std::fmt::Debug + 'a,
    {

        let mut pos_values = HashMap::new();

        let mut pos = 1;
        let len = pair_updater.len();
        for i in pair_updater.keys() {
            query.push_str(&format!(" {} = ${}", i, pos));
            pos_values.insert(pos, pair_updater.get(i).unwrap());
            if len > pos {
                query.push(',');
            }
            pos += 1
        }

        if let Some(condition) = condition {
            let len = condition.len() + pos - 1;
            for i in condition.keys() {
                pos_values.insert(pos, condition.get(i).unwrap());
                query.push_str(&format!(" {} = ${}", i, pos));
                if pos < len {
                    query.push_str(" AND");
                }
                pos += 1;
            }
        }

        let mut sql = sqlx::query(query);
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

        sql
    }

    pub fn delete<'a>(
        condition: Option<&'a HashMap<&'a str, TypeTable>>,
        query: &'a mut String,
    ) -> Query<'a, Postgres, PgArguments> {

        if let Some(condition) = condition {
            query.push_str(" WHERE");

            let mut pos_column = HashMap::new();
            let mut pos = 1;

            let len = condition.len();
            for t in condition.keys() {

                query.push_str(&format!(" {} = ${}", t, pos));
                pos_column.insert(pos, condition.get(t).unwrap());
                if pos < len {
                    query.push_str(" AND");
                }
                pos += 1;
            }

            let mut sql = sqlx::query(query);

            for i in 1..pos {
                sql = match pos_column.get(&i).unwrap() {
                    TypeTable::OptionCredential(e) => sql.bind(e),
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