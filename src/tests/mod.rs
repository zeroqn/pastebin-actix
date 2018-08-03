use actix::prelude::*;

use crate::models::executor::DatabaseExecutor as DbExecutor;
use crate::ENV;

#[macro_use]
pub mod common;
pub mod paste;
pub mod testdata;

lazy_static! {
    static ref TEST_DB_CHAN: Addr<DbExecutor> =
        common::connect_db(&ENV.database_url, ENV.actix_db_conns);
}
