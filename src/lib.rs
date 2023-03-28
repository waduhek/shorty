//! Shorty is a simple URL shortner. It will generate a short ID for a provided
//! long URL and it can be used with your hosting solution to redirect users to
//! your desired location. Once created, the URLs are immutable and only the
//! view count will be updated on every get request.
//!
//! # Usage
//!
//! ```rust,no_run
//! use shorty::{create_url, get_url, setup_db};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Call `setup_db` to setup any indexes required by the library.
//!     if let Err(_) = setup_db().await {
//!         panic!("Uh oh! Could not setup the DB")
//!     }
//!
//!     let long_url = "https://example.com";
//!
//!     let short_id = match create_url(long_url).await {
//!         Ok(id) => id,
//!         Err(_) => panic!("Uh oh! Could not shorten URL"),
//!     };
//!     println!("Generated short ID: {short_id}");
//!
//!     let full_url = match get_url(&short_id).await {
//!         Ok(url) => url,
//!         Err(_) => panic!("Uh oh! Could not lengthen URL"),
//!     };
//!
//!     // The provided short ID does not exist.
//!     if full_url.is_none() {
//!         panic!("That short ID was not found!");
//!     }
//!
//!     let full_url = full_url.unwrap();
//!     println!("Lengthened URL: {full_url}");
//! }
//! ```

mod db;
mod id;

use crate::{db::urls::Url, id::generate_id};

/// Sets up the database required for the library.
///
/// Sets up the URL collection where the shortened URL are stored. Call this
/// function atleast once before saving any URLs.
///
/// # Errors
///
/// A description of the error from MongoDB if the setup could not be performed.
///
/// # Examples
///
/// ```rust,no_run
/// # use shorty::setup_db;
/// #
/// # #[tokio::main]
/// # async fn main () {
/// if let Err(err) = setup_db().await {
///     panic!("could not setup DB due to: {}", err);
/// }
/// # }
/// ```
pub async fn setup_db() -> Result<(), String> {
    let setup_result = Url::setup().await;
    match setup_result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Creates a shortened URL for the provided full URL.
///
/// # Returns
///
/// The generated short ID for the full URL.
///
/// # Errors
///
/// Returns an error if a unique ID could not be generated for the full URL
///
/// # Examples
/// ```rust,no_run
/// # use shorty::create_url;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), &'static str> {
/// let short_id = create_url("https://example.com").await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_url(full_url: &str) -> Result<String, &'static str> {
    const SAVE_RETRY_COUNT: u8 = 2;

    let id = generate_id(full_url);
    let mut url_obj = Url::new(id, full_url, 0).await;

    for _ in 1..SAVE_RETRY_COUNT {
        match url_obj.save().await {
            Ok(_) => return Ok(url_obj.get_short_id().to_string()),
            Err(err) => {
                // An error should only really occur when the generated ID is
                // already present in the DB.
                println!("{err:#?}");
                url_obj.update_short_id(generate_id(full_url));
            }
        }
    }

    Err("could not generate a unique ID")
}

/// Gets the full URL stored against the provided short ID and updates it's view
/// count.
///
/// # Returns
///
/// The full URL stored against the short ID otherwise
/// [`None`](std::option::Option::None).
///
/// # Errors
///
/// The function will return an error if an error occurs at the DB layer.
///
/// # Examples
///
/// ```rust,no_run
/// # use shorty::get_url;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), &'static str> {
/// let full_url = get_url("abcd1234").await?;
/// match full_url {
///     Some(url) => println!("{url}"),
///     None => println!("url not found"),
/// };
/// # Ok(())
/// # }
/// ```
pub async fn get_url(short_id: &str) -> Result<Option<String>, &'static str> {
    let url_object = match Url::fetch_url(short_id).await {
        Ok(obj) => obj,
        Err(e) => {
            println!("error while fetching url: {e}");
            return Err("error while getting url object");
        }
    };

    if url_object.is_none() {
        return Ok(None);
    }

    let mut url_object = url_object.unwrap();
    url_object.increment_view_count();

    if let Err(e) = url_object.save().await {
        println!("error while updating view count: {e}");
        return Err("could not update URL view count");
    }

    Ok(Some(url_object.get_full_url().to_string()))
}
