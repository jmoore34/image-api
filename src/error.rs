use axum::http::StatusCode;

pub type ServerError = (StatusCode, String);