use std::collections::HashMap;

use crate::{database::utils::Repository, models::utils::*};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Row};

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub id: uuid::Uuid,
    pub role: Role,
}

pub fn verify_pass(pass: &[u8], pass_db: &str) -> Verify<bool> {
    match verify(pass, pass_db) {
        Ok(true) => Verify::Ok(true),
        _ => Verify::Unauthorized,
    }
}

pub fn encrypt(pass: &[u8]) -> Result<String, Error> {
    Ok(hash(pass, DEFAULT_COST)?)
}

pub fn create_token(user: &User) -> Result<String, Error> {
    let secret = std::env::var("SECRET_KEY")?;

    Ok(encode(
        &Header::default(),
        &Claims::from(user),
        &EncodingKey::from_secret(secret.as_ref()),
    )?)
}

pub fn verify_token(token: &str) -> Result<Verify<Claims>, Error> {
    let secret = std::env::var("SECRET_KEY")?;

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(e) => Ok(Verify::Ok(e.claims)),
        Err(_) => Ok(Verify::Unauthorized),
    }
}

pub async fn create_default_user(db: &impl Repository) -> Result<(), Error> {
    if db
        .get::<User>(Some(HashMap::from([("role", Role::Admin.into())])))
        .await
        .is_ok()
    {
        return Ok(());
    }

    let user = User {
        id: uuid::Uuid::new_v4(),
        username: std::env::var("IPAM_USER_ROOT").unwrap_or("admin".into()),
        password: encrypt(
            std::env::var("IPAM_PASSWORD_ROOT")
                .unwrap_or("admin".into())
                .as_ref(),
        )?,
        role: Role::Admin,
    };

    match db.insert::<User>(vec![user]).await {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::CreateDefaultUser),
    }
}

impl From<PgRow> for User {
    fn from(value: PgRow) -> Self {
        Self {
            id: value.get("id"),
            username: value.get("username"),
            password: value.get("password"),
            role: value.get("role"),
        }
    }
}

impl From<&User> for Claims {
    fn from(value: &User) -> Self {
        Self {
            exp: time::OffsetDateTime::now_utc().unix_timestamp() as usize,
            id: value.id,
            role: value.role.clone(),
        }
    }
}

pub enum Verify<T> {
    Ok(T),
    Unauthorized,
}

#[derive(Debug)]
pub enum Error {
    Encrypt,
    EncodeToken,
    CreateDefaultUser,
    SecretKey,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Encrypt => write!(f, "Encrypt Error"),
            Error::EncodeToken => write!(f, "Encode Token Error"),
            Error::SecretKey => write!(f, "Secret key not found"),
            Error::CreateDefaultUser => write!(f, "Create user default error"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::env::VarError> for Error {
    fn from(_value: std::env::VarError) -> Self {
        Self::SecretKey
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(_value: jsonwebtoken::errors::Error) -> Self {
        Self::EncodeToken
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(_value: bcrypt::BcryptError) -> Self {
        Self::Encrypt
    }
}

#[derive(Deserialize, Serialize, sqlx::Type, Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Guest,
    Operator,
}

impl Table for User {
    fn columns() -> Vec<&'static str> {
        vec!["id", "username", "password", "role"]
    }
    fn name() -> String {
        String::from("USERS")
    }

    fn query_insert() -> String {
        format!(
            "INSERT INTO {} (id, username, password, role) VALUES ($1, $2, $3, $4)",
            User::name()
        )
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![
            self.id.into(),
            self.username.into(),
            self.password.into(),
            self.role.into(),
        ]
    }
}
