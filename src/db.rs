pub mod urls;

use std::env;

use mongodb::{bson::doc, options::ClientOptions, Client, Database};

/// Gets a new connection to the DB used by the application.
///
/// # Example
///
/// ```rust,ignore
/// let db = get_shorty_db_connection().await?;
/// ```
async fn get_shorty_db_connection() -> mongodb::error::Result<Database> {
    let client_options = ClientOptions::parse(
        env::var("SHORTY_MONGODB_URI")
            .expect("could not find connection string"),
    )
    .await?;
    let client = Client::with_options(client_options)?;
    let database = client.database(
        &env::var("SHORTY_MONGODB_DATABASE")
            .expect("could not find database string"),
    );

    database.run_command(doc! { "ping": 1 }, None).await?;

    Ok(database)
}
