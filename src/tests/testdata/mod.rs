use actix::prelude::*;
use diesel;
use diesel::prelude::*;
use futures::Future;

use crate::common::error::ServerError;
use crate::models::{executor::DatabaseExecutor as DbExecutor, paste::*, schema::*};
use crate::services::paste as paste_srv;

use crate::tests::TEST_DB_CHAN;

pub fn create_test_paste_list() -> Vec<Paste> {
    use std::time::SystemTime;

    let db_chan = TEST_DB_CHAN.clone();
    let mut title_list: Vec<String> = (1..10)
        .map(|n| "test title ".to_owned() + &n.to_string())
        .collect();
    let mut body_list: Vec<String> = (1..10)
        .map(|n| "test body ".to_owned() + &n.to_string())
        .collect();
    let created_at = SystemTime::now();
    let mut paste_list: Vec<Paste> = vec![];

    for _ in 1..10 {
        let title = title_list.pop().unwrap();
        let body = body_list.pop().unwrap();
        paste_list.push(sync_send!(
            db_chan,
            paste_srv::CreatePasteMsg {
                title,
                body,
                created_at,
            }
        ));
    }

    paste_list
}

pub struct ClearDb();

impl Message for ClearDb {
    type Result = Result<usize, ServerError>;
}

impl Handler<ClearDb> for DbExecutor {
    type Result = Result<usize, ServerError>;

    fn handle(&mut self, _msg: ClearDb, _: &mut Self::Context) -> Self::Result {
        diesel::delete(pastes::table)
            .execute(&self.0.get().map_err(ServerError::R2d2)?)
            .map_err(ServerError::Database)
    }
}

pub fn clear() {
    let db_chan = TEST_DB_CHAN.clone();

    sync_send!(db_chan, ClearDb {});
}

pub fn recreate() -> Vec<Paste> {
    clear();
    create_test_paste_list()
}
