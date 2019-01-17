extern crate saphir;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
extern crate serde_urlencoded;
extern crate csv;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate r2d2;

mod controllers;
mod dataset;
mod mongo_connection;
mod models;

use saphir::*;
use self::controllers::LookupController;
use self::dataset::DATASET_LOCATION;
use self::mongo_connection::MongoConnection;
use self::models::RepositoryCollection;

fn main() {
    let mongo = MongoConnection::new("mongodb://localhost:27017").expect("Cannot start a lucid server without a database");
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

