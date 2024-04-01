use axum::{
    extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse,
};
use thiserror::Error;

use crate::common::response::AppJson;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("the requrest body container invalid Json")]
    JsonRejection(JsonRejection),

    #[error("server internal error {0}")]
    Other(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // todo  思考一下 这个局部结构体是否有必要放在这个位置

        let (status, maessage) = match self {
            AppError::JsonRejection(rejection) => {
                (rejection.status(), rejection.body_text())
            }
            AppError::Other(err) => {
                (StatusCode::OK, format!("internal server error: {}", err))
            }
        };
        (
            status,
            AppJson(crate::common::response::build_error_response(maessage)),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Other(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Other(anyhow::Error::new(err))
    }
}

impl From<walkdir::Error> for AppError {
    fn from(err: walkdir::Error) -> Self {
        AppError::Other(anyhow::Error::new(err))
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(err: zip::result::ZipError) -> Self {
        AppError::Other(anyhow::Error::new(err))
    }
}

impl From<std::path::StripPrefixError> for AppError {
    fn from(err: std::path::StripPrefixError) -> Self {
        AppError::Other(anyhow::Error::new(err))
    }
}
