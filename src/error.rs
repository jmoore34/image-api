
use std::io::Error;

use axum::{http::StatusCode, response::IntoResponse};
use migration::DbErr;
use sea_orm::TransactionError;

/// The main error type we use. This error type lets us easily specify a status
/// code (e.g. 400, 500, etc) as well as an additional string message.
pub struct ServerError {
    code: StatusCode,
    msg: String,
}


/// This trait implementation allow the Axum web framework to understand our
/// error type and easily create responses (with the proper status code) from it
impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        // Axum has built-in support for generating responses from tuples in the 
        // format (error code, response body), even for non-error error codes (such as 200).
        // Here we leverage that to easily create a response.
        (self.code, self.msg).into_response()
    }
}

/// Constructor to allow us to more concisely make ServerErrors where
/// needed in our server code.
impl ServerError {
    pub fn new(code: StatusCode, msg: String)  -> ServerError {
        ServerError { code, msg }
    }
}

// Database errors should automatically be converted to HTTP 500 Internal Server Errors.
// This is because these errors should not happen.
// (In contrast, whenever we select a single item from a table,
// we get an Option, not a Result. This means a missing item would
// not return a DbErr but rather a None, so we can gracefully return
// a 404 instead.)
impl From<DbErr> for ServerError {
    fn from(err: DbErr) -> Self {
        ServerError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Database error: {err}"),
        }
    }
}

/// Same idea as above: transaction errors are catastrophic events
/// that we don't expect (unlike 400-class errors), so we give
/// a HTTP 500 error
impl From<TransactionError<DbErr>> for ServerError {
    fn from(err: TransactionError<DbErr>) -> Self {
        ServerError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Transaction error: {err}"),
        }
    }
}

/// The following allows us to use the `?` operator on results that
/// we don't expect to fail (and hence should be 500 errors). This 
/// is preferable to panicking.
impl From<Error> for ServerError {
    fn from(err: Error) -> Self {
        ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, 
        format!("Error: {err}"))
    }
}

/// Used for unusual HTTP client errors.
impl From<ureq::Error> for ServerError {
    fn from(err: ureq::Error) -> Self {
        ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, 
        format!("Error while making request: ureq: {err}"))
    }
}