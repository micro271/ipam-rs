use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Serialize, de::DeserializeOwned};

const ALGORITHM_JWT: Algorithm = Algorithm::HS256;

pub trait Claim: std::fmt::Debug {}

pub fn verify_passwd<T: AsRef<[u8]>>(pass: T, pass_db: &str) -> bool {
    verify(pass.as_ref(), pass_db).unwrap_or(false)
}

pub fn encrypt<T: AsRef<[u8]>>(pass: T) -> Result<String, error::Error> {
    Ok(hash(pass.as_ref(), DEFAULT_COST)?)
}

pub fn create_token<T>(claim: T) -> Result<String, error::Error>
where
    T: Serialize + Claim,
{
    let secret = std::env::var("SECRET_KEY")?;

    Ok(encode(
        &Header::new(ALGORITHM_JWT),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )?)
}

pub fn verify_token<T, B: AsRef<str>>(token: B) -> Result<T, error::Error>
where
    T: DeserializeOwned + Claim,
{
    let secret = std::env::var("SECRET_KEY")?;

    match decode(
        token.as_ref(),
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(ALGORITHM_JWT),
    ) {
        Ok(e) => Ok(e.claims),
        Err(e) => Err(e.into()),
    }
}

pub mod error {
    #[derive(Debug)]
    pub enum Error {
        Encrypt,
        EncodeToken,
        SecretKey,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::Encrypt => write!(f, "Encrypt Error"),
                Error::EncodeToken => write!(f, "Encode Token Error"),
                Error::SecretKey => write!(f, "Secret key not found"),
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
}
