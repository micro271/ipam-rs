use crate::{
    database::repository::{Repository, error::RepositoryError},
    models::user::{Role, User},
};
use libipam::services::authentication::{Claim, encrypt};
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
    if let Ok(e) = db
        .get::<User>(
            Some(std::collections::HashMap::from([(
                "role",
                Role::Admin.into(),
            )])),
            None,
            None,
        )
        .await
        .map(|x| x.take_data().unwrap().remove(0))
    {
        tracing::info!(
            "The admin user already exists [ username: {}, password: {}, create_at: {:?} ]",
            e.username,
            e.password,
            e.create_at
        );
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
    let username = user.username.clone();
    let pass = user.password.clone();
    let role = user.role.clone();

    db.insert::<User>(user).await?;

    tracing::info!(
        "We've created the admin user [ username: {}, password: {}, role: {:?} ]",
        username,
        pass,
        role
    );
    Ok(())
}

impl From<User> for Claims {
    fn from(value: User) -> Self {
        Self {
            exp: usize::try_from(
                (time::OffsetDateTime::now_utc() + time::Duration::hours(6)).unix_timestamp(),
            )
            .unwrap(),
            id: value.id,
            role: value.role,
        }
    }
}
