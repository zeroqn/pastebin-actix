use actix::prelude::*;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub struct DatabaseExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DatabaseExecutor {
    type Context = SyncContext<Self>;
}
