//! `CkError` as an HTTP response: a status code plus the error as compact JSON.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use ck_core::{CkError, ErrorCode};

pub struct ApiError(pub CkError);

impl From<CkError> for ApiError {
    fn from(e: CkError) -> Self {
        Self(e)
    }
}

pub fn status_of(code: ErrorCode) -> StatusCode {
    match code {
        ErrorCode::Auth => StatusCode::FORBIDDEN,
        ErrorCode::NotFound => StatusCode::NOT_FOUND,
        ErrorCode::Request => StatusCode::BAD_REQUEST,
        ErrorCode::PatchStale => StatusCode::CONFLICT,
        ErrorCode::LockHeld | ErrorCode::LockNotHeld => StatusCode::LOCKED,
        ErrorCode::Scope => StatusCode::FORBIDDEN,
        ErrorCode::Budget => StatusCode::TOO_MANY_REQUESTS,
        ErrorCode::Render => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::PatchInvalid | ErrorCode::Schema | ErrorCode::Media | ErrorCode::Voice => {
            StatusCode::UNPROCESSABLE_ENTITY
        }
        ErrorCode::Stage | ErrorCode::Gates | ErrorCode::Retry | ErrorCode::HistoryEmpty => {
            StatusCode::CONFLICT
        }
        ErrorCode::Storage
        | ErrorCode::Migration
        | ErrorCode::Server
        | ErrorCode::OutputCap
        | ErrorCode::Asset => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (status_of(self.0.code), Json(self.0)).into_response()
    }
}
