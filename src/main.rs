#![feature(rust_2018_preview)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure_derive;

mod apps;
mod common;
mod controllers;
mod models;
mod services;
#[cfg(test)]
mod tests;

use actix::prelude::*;
use actix_web::server;
// prelude is required for PgConnection::establish()
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

use crate::models::executor::DatabaseExecutor;

lazy_static! {
    static ref ENV: common::env::Env = common::env::load();
}

pub struct State {
    db_chan: Addr<DatabaseExecutor>,
}

fn main() {
    let sys = actix::System::new("pastebin-actix");

    let manager = ConnectionManager::<PgConnection>::new(ENV.database_url.clone());
    let pool = Pool::builder()
        .build(manager)
        .expect("cannot build database connection pool");
    let addr = SyncArbiter::start(ENV.actix_db_conns, move || DatabaseExecutor(pool.clone()));

    server::new(move || {
        apps::paste::create(State {
            db_chan: addr.clone(),
        })
    }).bind(&ENV.bind_addr)
    .unwrap()
    .run();

    sys.run();
}
