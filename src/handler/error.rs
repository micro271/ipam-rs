use libipam::response_error::ResponseError;
use axum::http::StatusCode;
use crate::database::repository::error::RepositoryError;

impl From<RepositoryError> for ResponseError {
    fn from(value: RepositoryError) -> Self {
        let builder = ResponseError::builder();

        let builder = match value {
            RepositoryError::Sqlx(e) => {
                builder
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .title("Database error".to_string())
                    .detail(e.to_string())
            },
            RepositoryError::RowNotFound => {
                builder
                    .status(StatusCode::BAD_REQUEST)
                    .title("Row not found".to_string())
            },
            RepositoryError::ColumnNotFound(e) => {
                builder
                    .status(StatusCode::BAD_REQUEST)
                    .title(e.to_string())
            }
        };

        builder.build()
    }
}