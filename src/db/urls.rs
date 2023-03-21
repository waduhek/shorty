use mongodb::{
    bson::doc,
    error::Result as MongoResult,
    options::{IndexOptions, UpdateModifications},
    results::{InsertOneResult, UpdateResult},
    Collection, IndexModel,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Url {
    /// A short ID for the URL.
    short_id: String,
    /// The full URL for this short.
    full_url: String,
    /// Number of times this link was accessed.
    view_count: u32,
}

impl Url {
    /// Creates a new `Url`.
    pub fn new(short_id: String, full_url: &str, view_count: u32) -> Self {
        Url {
            short_id,
            full_url: full_url.to_string(),
            view_count,
        }
    }

    /// Gets a reference to the short ID of the current URL.
    pub fn get_short_id(&self) -> &str {
        &self.short_id
    }

    pub fn get_full_url(&self) -> &str {
        &self.full_url
    }

    /// Updates the short ID of the current URL.
    pub(crate) fn update_short_id(&mut self, new_id: String) {
        self.short_id = new_id;
    }

    /// Gets the MongoDB collection for the URLs.
    async fn get_collection() -> MongoResult<Collection<Self>> {
        let db = super::get_shorty_db_connection().await?;
        Ok(db.collection::<Self>("urls"))
    }

    pub(crate) async fn setup() -> MongoResult<()> {
        let url_collection = Self::get_collection().await?;

        // Set index on the `short_id` field.
        let short_id_index = IndexModel::builder()
            .keys(doc! { "short_id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let res = url_collection.create_index(short_id_index, None).await?;
        println!("{res:?}");
        Ok(())
    }
}

/// Saves a new URL to the DB.
///
/// # Example
///
/// ```rust,ignore
/// let url = Url::new("abcde", "https://example.com", 0);
/// let insert_result = save_url(url);
/// ```
pub(crate) async fn save_url(url: &Url) -> MongoResult<InsertOneResult> {
    let url_collection = Url::get_collection().await?;
    url_collection.insert_one(url, None).await
}

/// Gets the URL against the provided `short_id`.
///
/// # Example
///
/// ```rust,ignore
/// let url: Option<Url> = get_url("abcd").await?;
/// ```
pub(crate) async fn get_url(short_id: &str) -> MongoResult<Option<Url>> {
    let url_collection = Url::get_collection().await?;
    let fetched_url = url_collection
        .find_one(doc! { "short_id": short_id }, None)
        .await?;

    Ok(fetched_url)
}

/// Increments the view count of the URL with the provided short ID.
///
/// # Example
///
/// ```rust,ignore
/// let short_id = "abcd123";
/// let new_view_count = 10;
///
/// let update_result = increment_url_view_count(&short_id);
/// ```
pub(crate) async fn increment_url_view_count(
    short_id: &str,
) -> MongoResult<UpdateResult> {
    let url_collection = Url::get_collection().await?;
    let update_result = url_collection
        .update_one(
            doc! { "short_id": short_id },
            UpdateModifications::Document(doc! { "$inc": { "view_count": 1 } }),
            None,
        )
        .await?;

    Ok(update_result)
}
