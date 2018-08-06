use failure::Error;

use actix::{prelude::*, SystemRunner};
use actix_web::server;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

use crate::common::config::Config;
use crate::models::executor::DatabaseExecutor;

pub struct State {
    pub db_chan: Addr<DatabaseExecutor>,
}

pub struct Server {
    runner: SystemRunner,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: &Config) -> Result<Self, Error> {
        let database_url = format!(
            "postgres://{}:{}@{}/{}",
            config.postgres.username,
            config.postgres.password,
            config.postgres.host,
            config.postgres.database,
        );

        let runner = actix::System::new("pastebin-actix");

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("cannot build database connection pool");
        let addr = SyncArbiter::start(config.actix.connections, move || {
            DatabaseExecutor(pool.clone())
        });

        let server = server::new(move || {
            crate::apps::paste::create(State {
                db_chan: addr.clone(),
            })
        });
        let server_url = format!("{}:{}", config.server.ip, config.server.port);

        server.bind(server_url)?.start();

        Ok(Server { runner })
    }

    pub fn start(self) -> i32 {
        self.runner.run()
    }
}
