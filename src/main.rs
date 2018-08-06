#![feature(rust_2018_preview)]

#[macro_use]
extern crate diesel;
#[cfg(test)]
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
mod server;
mod services;
#[cfg(test)]
mod tests;

use std::process::exit;

use crate::common::{config::Config, constant::CONFIG_FILENAME};
use crate::server::Server;

fn main() {
    let config = Config::load(CONFIG_FILENAME);
    let server = Server::new(&config).unwrap();

    exit(server.start());
}
