use macros::MapQuery;
use time::OffsetDateTime;

use super::{Deserialize, FromPgRow, Serialize, Table, Updatable, Uuid};

#[derive(Deserialize, Serialize, Debug, Clone, Table, FromPgRow)]
#[table_name("users")]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub role: Role,
    pub is_active: bool,

    #[offset_timestamp((-3,0,0))]
    pub create_at: time::OffsetDateTime,

    #[offset_timestamp((-3,0,0))]
    pub last_login: Option<time::OffsetDateTime>,
}

#[derive(Debug, MapQuery, Default)]
pub struct UserCondition {
    pub id: Option<Uuid>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
    pub is_active: Option<bool>,
}

impl UserCondition {
    pub fn p_key(id: Uuid) -> Self {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }
    pub fn role(role: Role) -> Self {
        Self {
            role: Some(role),
            ..Default::default()
        }
    }
    pub fn username(username: String) -> Self {
        Self {
            username: Some(username),
            ..Default::default()
        }
    }
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

#[derive(Deserialize, Serialize, sqlx::Type, Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Guest,
    Operator,
}

impl std::cmp::PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.username.eq(&other.username)
    }
}

#[derive(Debug, Deserialize, Updatable, Default)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
    pub last_login: Option<OffsetDateTime>,
}
