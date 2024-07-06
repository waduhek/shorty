use bson::ser::to_document;
use chrono::{DateTime, Utc};
use mongodb::{
    bson::doc,
    error::Result as MongoResult,
    options::{IndexOptions, UpdateModifications},
    Collection, IndexModel,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct UrlModel {
    /// A short ID for the URL.
    short_id: String,
    /// The full URL for this short.
    full_url: String,
    /// Number of times this link was accessed.
    view_count: u32,

    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    updated_at: DateTime<Utc>,
}

/// The changes that can be performed on the `UrlModel` struct.
#[derive(Debug, Serialize)]
struct UrlModelChangeset {
    #[serde(skip_serializing_if = "Option::is_none")]
    short_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    full_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    view_count: Option<u32>,
}

impl From<UrlModelChangeset> for UpdateModifications {
    fn from(value: UrlModelChangeset) -> Self {
        let mut serialised = to_document(&value)
            .expect("could not serialize changeset for update");
        serialised.insert("updated_at", Utc::now());

        Self::Document(doc! {
            "$set": serialised,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Url {
    /// The current URL stored.
    model: UrlModel,
    /// The collection of the model.
    collection: Collection<UrlModel>,
    /// Changes to be applied to the model.
    changeset: Option<UrlModelChangeset>,
    /// Set if the current instance was fetched from the DB.
    is_fetched_from_db: bool,
}

impl Url {
    /// Creates a new `Url`.
    pub async fn new(
        short_id: String,
        full_url: &str,
        view_count: u32,
    ) -> Self {
        let collection = match Self::get_collection().await {
            Ok(coll) => coll,
            Err(err) => {
                panic!("could not get a collection for URLs: {err}");
            }
        };

        let model = UrlModel {
            short_id,
            full_url: full_url.to_string(),
            view_count,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Url {
            model,
            collection,
            changeset: None,
            is_fetched_from_db: false,
        }
    }

    /// Constructs a new instance of `Url` from a `UrlModel`.
    fn from_model(model: UrlModel, collection: Collection<UrlModel>) -> Self {
        Url {
            model,
            collection,
            changeset: None,
            is_fetched_from_db: true,
        }
    }

    async fn _save_from_changeset(&mut self) -> MongoResult<()> {
        let changeset = self
            .changeset
            .take()
            .expect("trying to save from changeset when changeset is None");

        self.collection
            .update_one(doc! { "short_id": &self.model.short_id }, changeset)
            .await?;
        Ok(())
    }

    async fn _save_new_url(&self) -> MongoResult<()> {
        if !self.is_fetched_from_db {
            self.collection.insert_one(&self.model).await?;
        }
        Ok(())
    }

    /// Saves the current model to the database.
    ///
    /// If any changes were made to the data stored in the model, saves only
    /// those. If a new instance was created, creates a new document in the
    /// database.
    pub async fn save(&mut self) -> MongoResult<()> {
        match self.changeset {
            Some(_) => self._save_from_changeset().await,
            None => self._save_new_url().await,
        }
    }

    /// Increments the view count of the URL. Calling this function multiple
    /// times without calling [`save`](Url::save) will have no effect on the
    /// incremented view count.
    pub fn increment_view_count(&mut self) {
        self.changeset = Some(UrlModelChangeset {
            short_id: None,
            full_url: None,
            view_count: Some(self.model.view_count + 1),
        });
    }

    /// Fetches a URL with the provided short ID.
    pub async fn fetch_url(short_id: &str) -> MongoResult<Option<Self>> {
        let url_collection = Self::get_collection().await?;

        let fetched_url = url_collection
            .find_one(doc! { "short_id": short_id })
            .await?;

        match fetched_url {
            Some(url_model) => {
                Ok(Some(Url::from_model(url_model, url_collection)))
            }
            None => Ok(None),
        }
    }

    /// Gets a reference to the short ID of the current URL.
    pub fn get_short_id(&self) -> &str {
        &self.model.short_id
    }

    /// Gets the full URL of the current document.
    pub fn get_full_url(&self) -> &str {
        &self.model.full_url
    }

    /// Updates the short ID of the current URL.
    pub fn update_short_id(&mut self, new_id: String) {
        match self.changeset.take() {
            Some(change) => {
                self.changeset = Some(UrlModelChangeset {
                    short_id: Some(new_id),
                    full_url: change.full_url,
                    view_count: change.view_count,
                });
            }
            None => {
                self.changeset = Some(UrlModelChangeset {
                    short_id: Some(new_id),
                    full_url: None,
                    view_count: None,
                });
            }
        };
    }

    /// Gets the MongoDB collection for the URLs.
    async fn get_collection() -> MongoResult<Collection<UrlModel>> {
        let db = super::get_shorty_db_connection().await?;
        Ok(db.collection::<UrlModel>("urls"))
    }

    /// Sets up the index required by the `Url` model.
    pub async fn setup() -> MongoResult<()> {
        let url_collection = Self::get_collection().await?;

        // Set index on the `short_id` field.
        let short_id_index = IndexModel::builder()
            .keys(doc! { "short_id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        url_collection.create_index(short_id_index).await?;
        Ok(())
    }
}
