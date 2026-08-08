use axum::{http::StatusCode, response::IntoResponse};
use watchdog_core::RepoError;

use crate::dto::endpoints::ValidationError;

pub enum ApiError {
    Repo(RepoError),
    Validation(ValidationError),
}

impl From<RepoError> for ApiError {
    fn from(repo: RepoError) -> Self {
        Self::Repo(repo)
    }
}

impl From<ValidationError> for ApiError {
    fn from(validation: ValidationError) -> Self {
        Self::Validation(validation)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Repo(repo_error) => match repo_error {
                RepoError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
                RepoError::Database(log) => {
                    tracing::error!("{log}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )
                }
            },
            ApiError::Validation(validation) => (StatusCode::BAD_REQUEST, validation.to_string()),
        }
        .into_response()
    }
}
