use crate::database::utils::RepositoryError;
use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

pub enum ResponseError {
    DataNotFound,
    Unauthorized,
    ServerError,
    ColumnNotFound(Option<String>),
    StatusCode(StatusCode),
    Custom { body: String, status: StatusCode },
}

impl From<RepositoryError> for ResponseError {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::Sqlx(_) => Self::ServerError,
            RepositoryError::RowNotFound => Self::DataNotFound,
            RepositoryError::ColumnNotFound(e) => Self::ColumnNotFound(e),
        }
    }
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::DataNotFound => write!(f, "Data not found"),
            ResponseError::Unauthorized => write!(f, "User not authorized"),
            ResponseError::ServerError => write!(f, "Internal server error"),
            ResponseError::ColumnNotFound(e) => match e {
                Some(e) => write!(f, "Column {e} not found"),
                None => write!(f, "Column not found"),
            },
            ResponseError::StatusCode(status_code) => write!(f, "{status_code}"),
            ResponseError::Custom { body, status } => write!(f, "({status}) {body}"),
        }
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> axum::response::Response {
        let (body, status) = match self {
            ResponseError::DataNotFound => (
                json!({"Error": Self::DataNotFound.to_string()}).to_string(),
                StatusCode::NOT_FOUND,
            ),
            ResponseError::Unauthorized => (
                json!({"Error": Self::Unauthorized.to_string()}).to_string(),
                StatusCode::UNAUTHORIZED,
            ),
            ResponseError::ServerError => (
                json!({"Error": Self::ServerError.to_string()}).to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            ResponseError::ColumnNotFound(e) => (
                json!({"Error": "Column not found", "Column":e}).to_string(),
                StatusCode::NOT_FOUND,
            ),
            ResponseError::StatusCode(e) => ("".to_string(), e),
            ResponseError::Custom { body, status } => (body, status),
        };

        Response::builder()
            .status(status)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap_or_default()
            .into_response()
    }
}
