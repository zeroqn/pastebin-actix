use std::{fmt, error::Error as StdError, num::ParseIntError};

use actix::MailboxError;
use actix_web::{HttpResponse,
                error::{JsonPayloadError, PayloadError, ResponseError as ActixResponseError},
                http::StatusCode};
use diesel::result::Error as DieselError;
use futures::future::{self, Future};

use crate::common::constant;

#[derive(Debug)]
pub enum Error {
    PayloadError(String),
    DatabaseError(DieselError),
    BadID(ParseIntError),
    Custom(u16, String),
}

impl Error {
    pub fn bad_request(msg: &str) -> Self {
        Error::Custom(StatusCode::BAD_REQUEST.as_u16(), msg.to_owned())
    }
}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Error {
        Error::DatabaseError(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::BadID(err)
    }
}

impl From<MailboxError> for Error {
    fn from(_err: MailboxError) -> Error {
        Error::Custom(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "internal server error".to_string(),
        )
    }
}

impl From<PayloadError> for Error {
    fn from(err: PayloadError) -> Error {
        Error::PayloadError(err.to_string())
    }
}

impl From<JsonPayloadError> for Error {
    fn from(err: JsonPayloadError) -> Error {
        Error::PayloadError(err.to_string())
    }
}

impl From<Error> for Box<Future<Item = HttpResponse, Error = Error>> {
    fn from(err: Error) -> Box<Future<Item = HttpResponse, Error = Error>> {
        Box::new(future::err(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::PayloadError(ref msg) => msg.fmt(f),
            Error::DatabaseError(ref inner) => inner.fmt(f),
            Error::BadID(_) => write!(f, "{}", constant::ERR_MSG_BAD_ID),
            Error::Custom(ref code, ref msg) => write!(f, "{{ code: {}, msg: {} }}", code, msg),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::PayloadError(ref msg) => msg,
            Error::DatabaseError(ref err) => err.description(),
            Error::BadID(ref err) => err.description(),
            Error::Custom(.., ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::PayloadError(_) => None,
            Error::DatabaseError(ref err) => Some(err),
            Error::BadID(ref err) => Some(err),
            Error::Custom(_, _) => None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseError {
    pub code: u16,
    pub msg: String,
}

impl ActixResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let res_err = match *self {
            Error::PayloadError(ref msg) => ResponseError {
                code: StatusCode::BAD_REQUEST.as_u16(),
                msg: msg.to_string(),
            },
            Error::DatabaseError(ref err) => match err {
                &DieselError::NotFound => ResponseError {
                    code: StatusCode::NOT_FOUND.as_u16(),
                    msg: constant::ERR_MSG_DATA_NOT_FOUND.to_string(),
                },
                _ => ResponseError {
                    code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    msg: constant::ERR_MSG_DATABASE_OPEARTION_FAIL.to_string(),
                },
            },
            Error::BadID(_) => ResponseError {
                code: StatusCode::BAD_REQUEST.as_u16(),
                msg: self.to_string(),
            },
            Error::Custom(ref code, ref msg) => ResponseError {
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
