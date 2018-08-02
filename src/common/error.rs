use std::num::ParseIntError;

use actix::MailboxError;
use actix_web::{
    error::{JsonPayloadError, PayloadError, ResponseError as ActixResponseError},
    http::StatusCode,
    HttpResponse,
};
use diesel::result::Error as DieselError;

use crate::common::constant;

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "database error")]
    Database(#[cause] DieselError),
    #[fail(display = "actor mailbox error")]
    MailBox(#[cause] MailboxError),
}

#[derive(Debug, Fail)]
pub enum UserError {
    #[fail(display = "an internal error occurred. please try again later")]
    InternalError,
    #[fail(display = "bad payload: {}", _0)]
    PayloadError(String),
    #[fail(display = "bad id")]
    BadID(#[cause] ParseIntError),
    #[fail(display = "data not found")]
    NotFound,
    #[fail(display = "code: {}, msg: {}", code, msg)]
    Custom { code: u16, msg: String },
}

impl UserError {
    pub fn bad_request(msg: &str) -> Self {
        UserError::Custom {
            code: StatusCode::BAD_REQUEST.as_u16(),
            msg: msg.to_owned(),
        }
    }
}

impl From<ServerError> for UserError {
    fn from(err: ServerError) -> Self {
        match err {
            ServerError::Database(ref cause) => match cause {
                &DieselError::NotFound => UserError::NotFound,
                _ => UserError::InternalError,
            },
            _ => UserError::InternalError,
        }
    }
}

impl From<ParseIntError> for UserError {
    fn from(err: ParseIntError) -> Self {
        UserError::BadID(err)
    }
}

impl From<PayloadError> for UserError {
    fn from(err: PayloadError) -> Self {
        UserError::PayloadError(err.to_string())
    }
}

impl From<JsonPayloadError> for UserError {
    fn from(err: JsonPayloadError) -> Self {
        UserError::PayloadError(err.to_string())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseError {
    pub code: u16,
    pub msg: String,
}

impl ActixResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        let res_err = match *self {
            UserError::InternalError => ResponseError {
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                msg: self.to_string(),
            },
            UserError::PayloadError(ref msg) => ResponseError {
                code: StatusCode::BAD_REQUEST.as_u16(),
                msg: msg.to_string(),
            },
            UserError::BadID(_) => ResponseError {
                code: StatusCode::BAD_REQUEST.as_u16(),
                msg: self.to_string(),
            },
            UserError::NotFound => ResponseError {
                code: StatusCode::NOT_FOUND.as_u16(),
                msg: self.to_string(),
            },
            UserError::Custom { ref code, ref msg } => ResponseError {
                code: *code,
                msg: msg.to_string(),
            },
        };

        let status_code =
            StatusCode::from_u16(res_err.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        HttpResponse::build(status_code)
            .content_type(constant::CONTENT_TYPE_JSON)
            .json(res_err)
    }
}
