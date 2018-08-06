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

use crate::common::{config::Config, constant::CONFIG_FILENAME};
use crate::models::executor::DatabaseExecutor;

pub struct State {
    db_chan: Addr<DatabaseExecutor>,
}

fn main() {
    let config = Config::load(CONFIG_FILENAME);
    let database_url = format!(
        "postgres://{}:{}@{}/{}",
        config.postgres.username,
        config.postgres.password,
        config.postgres.host,
        config.postgres.database,
    );
    let server_url = format!("{}:{}", config.server.ip, config.server.port);

    let sys = actix::System::new("pastebin-actix");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("cannot build database connection pool");
    let addr = SyncArbiter::start(config.actix.connections, move || {
        DatabaseExecutor(pool.clone())
    });

    server::new(move || {
        apps::paste::create(State {
            db_chan: addr.clone(),
        })
    }).bind(server_url)
    .unwrap()
    .run();

    sys.run();
}
