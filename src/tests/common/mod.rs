use std::sync::mpsc::channel;
use std::thread;

use actix::{self, prelude::*};
// prelude is required for PgConnection::establish()
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

use crate::models::executor::DatabaseExecutor;

#[macro_use]
pub mod macros;

pub fn connect_db(database_url: &'static str, conns: usize) -> Addr<DatabaseExecutor> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        let sys = actix::System::new("pastebin-test");

        let manager = ConnectionManager::<PgConnection>::new(database_url.to_owned());
        let pool = Pool::builder()
            .build(manager)
            .expect("cannot build database connection pool");
        let addr = SyncArbiter::start(conns, move || DatabaseExecutor(pool.clone()));

        tx.send(addr).unwrap();
        sys.run();
    });

    rx.recv().unwrap()
}
