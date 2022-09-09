use axum::{http::StatusCode, response::IntoResponse};
use migration::DbErr;

//pub type ServerError = (StatusCode, String);
pub struct ServerError {
    code: StatusCode,
    msg: String,
}

// Database errors should automatically be HTTP 500 Internal Server Errors
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

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        (self.code, self.msg).into_response()
    }
}

impl ServerError {
    pub fn new(code: StatusCode, msg: String)  -> ServerError {
        ServerError { code, msg }
    }
}
