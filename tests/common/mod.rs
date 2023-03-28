use std::env;

use dotenv;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client,
};

use shorty::setup_db;

/// Sets up the environment variables and the DB for the test.
///
/// # Panics
///
/// The database could not be setup.
pub async fn setup() {
    dotenv::from_filename("test.env").ok();
    if let Err(_) = setup_db().await {
        panic!("could not setup DB");
    };
}

/// Deletes the provided short ID from the DB.
pub async fn delete_by_short_id(short_id: String) {
    let options = ClientOptions::parse(env::var("SHORTY_MONGODB_URI").unwrap())
        .await
        .expect("could not create a client options");
    let client =
        Client::with_options(options).expect("could not create client");
    let database =
        client.database(&env::var("SHORTY_MONGODB_DATABASE").unwrap());

    let urls_collection = database.collection::<Document>("urls");
    urls_collection
        .delete_many(doc! { "short_id": short_id }, None)
        .await
        .expect("could not empty urls collection");
}
