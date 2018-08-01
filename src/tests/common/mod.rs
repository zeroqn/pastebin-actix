use std::sync::mpsc::channel;
use std::thread;

use actix;
use actix::prelude::*;

use crate::db::{self, executor::DbExecutor};

#[macro_use]
pub mod macros;

pub fn connect_db(database_url: &'static str, conns: usize) -> Addr<DbExecutor> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        let sys = actix::System::new("pastebin-test");
        let addr = SyncArbiter::start(conns, move || {
            DbExecutor::new(db::establish_connection(database_url))
        });
        tx.send(addr).unwrap();
        sys.run();
    });

    rx.recv().unwrap()
}
