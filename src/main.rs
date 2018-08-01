#![feature(rust_2018_preview)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod apps;
mod common;
mod controllers;
mod models;
mod services;
#[cfg(test)]
mod tests;

use actix::prelude::*;
use actix_web::server;

use crate::common::db::{self, executor::DbExecutor};

lazy_static! {
    static ref ENV: common::env::Env = common::env::load();
}

#[cfg(test)]
lazy_static! {
    static ref TEST_DB_CHAN: Addr<DbExecutor> =
        tests::common::connect_db(&ENV.database_url, ENV.actix_db_conns);
}

pub struct State {
    db_chan: Addr<DbExecutor>,
}

fn main() {
    let sys = actix::System::new("pastebin-actix");

    let addr = SyncArbiter::start(ENV.actix_db_conns, || {
        DbExecutor::new(db::establish_connection(&ENV.database_url))
    });

    server::new(move || {
        apps::paste::create(State {
            db_chan: addr.clone(),
        })
    }).bind(&ENV.bind_addr)
    .unwrap()
    .run();

    sys.run();
}
