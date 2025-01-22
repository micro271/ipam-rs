use super::*;

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

#[derive(Debug, Deserialize, Updatable)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
}
