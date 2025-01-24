use crate::{
    database::repository::{error::RepositoryError, Repository},
    models::user::*,
};
use libipam::authentication::{encrypt, Claim};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub id: uuid::Uuid,
    pub role: Role,
}

impl Claim for Claims {}

pub async fn create_default_user(db: &impl Repository) -> Result<(), RepositoryError> {
    if db
        .get::<User>(Some(std::collections::HashMap::from([(
            "role",
            Role::Admin.into(),
        )])), None, None)
        .await.is_ok()
    {
        return Ok(());
    }

    let user = User {
        id: Uuid::new_v4(),
        username: std::env::var("IPAM_USER_ROOT").unwrap_or("admin".into()),
        password: encrypt(std::env::var("IPAM_PASSWORD_ROOT").unwrap_or("admin".into())).unwrap(),
        role: Role::Admin,
        create_at: time::OffsetDateTime::now_utc(),
        is_active: true,
        last_login: None,
    };
    db.insert::<User>(user).await?;
    Ok(())
}

impl From<User> for Claims {
    fn from(value: User) -> Self {
        Self {
            exp: (time::OffsetDateTime::now_utc() + time::Duration::hours(6)).unix_timestamp()
                as usize,
            id: value.id,
            role: value.role,
        }
    }
}
