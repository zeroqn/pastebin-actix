use actix_web::HttpResponse;
use futures::future::Future;

use crate::common::error::UserError;

type FutureJsonResponse = Box<Future<Item = HttpResponse, Error = UserError>>;

#[macro_use]
pub mod macros;
pub mod paste;
