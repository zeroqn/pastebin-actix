use actix_web::HttpResponse;
use futures::future::Future;

use common::error::Error;

type FutureJsonResponse = Box<Future<Item = HttpResponse, Error = Error>>;

#[macro_use]
pub mod macros;
pub mod paste;
