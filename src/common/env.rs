use dotenv::dotenv;
use std::env;

pub struct Env {
    pub database_url: String,
    pub bind_addr: String,
    pub actix_db_conns: usize,
}

pub fn load() -> Env {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bind_addr = env::var("BIND_ADDR").expect("BIND_ADDR must be set");
    let actix_db_conns: usize = env::var("ACTIX_DB_CONNECTIONS")
        .expect("ACTIX_DB_CONNECTIONS must be set")
        .parse::<usize>()
        .expect("ACTIX_DB_CONNECTIONS must be a integer");

    Env {
        database_url,
        bind_addr,
        actix_db_conns,
    }
}
