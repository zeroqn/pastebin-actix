extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod models;
mod services;
mod controllers;
mod apps;
mod common;
#[cfg(test)]
mod tests;

use actix_web::server;
use actix::prelude::*;

use common::db::{self, executor::DbExecutor};

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
