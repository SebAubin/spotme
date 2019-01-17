extern crate saphir;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
extern crate serde_urlencoded;
extern crate csv;
#[macro_use]
extern crate lazy_static;
#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate r2d2;
extern crate toml;
extern crate config;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate serde_yaml;

mod controllers;
mod dataset;
mod mongo_connection;
mod models;
mod settings;

use env_logger::Builder;
use log::LevelFilter;
use std::env;
use saphir::*;
use self::controllers::LookupController;
use self::dataset::DATASET_LOCATION;
use self::mongo_connection::MongoConnection;
use self::models::RepositoryCollection;

fn main() {

    let config = settings::Settings::load().expect("Configuration errors are fatal");

    let mut builder = Builder::new();
    builder.filter(None, config.level_filter());
    builder.filter(Some("tokio_io"), LevelFilter::Off);
    builder.filter(Some("tokio_core"), LevelFilter::Off);
    builder.filter(Some("tokio_reactor"), LevelFilter::Off);
    builder.filter(Some("tokio_threadpool"), LevelFilter::Off);
    builder.filter(Some("mio"), LevelFilter::Off);
    builder.filter(Some("hyper"), LevelFilter::Off);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }

    builder.init();

    let mongo = MongoConnection::new(&config.server.mongo_uri).expect("Cannot start a spotme server without a database");
    let mut repos = RepositoryCollection::new(mongo.clone());
    println!("Loading repositories..");
    repos.load_repositories().expect("Lucid cannot start without fully initializing its repos");

    let server_builder = Server::builder().configure_router(|router| {
        let lookup = LookupController::new(repos.clone());
        router.add(lookup)
    }).configure_listener(|list_config| {
        list_config.set_uri("http://0.0.0.0:7974")
    }).build();

    println!("Server listening on port 7974..");
    let _ = server_builder.run();
}

