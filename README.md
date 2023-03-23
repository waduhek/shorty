# Shorty

A simple URL shortening library written in Rust. The library can be used with
your hosting solution to redirect users to the desired URL.

# Installation

To install the crate run

```cargo add --git https://github.com/waduhek/shorty```

# Usage

## Database setup

The database used by `shorty` will have to be setup. So before proceeding any
further, make sure you have the environment variables mentioned in
[`sample.env`](./sample.env) available. Also make sure that the database is
created in you MongoDB instance.

Now, run the following function at least once to setup the indexes and the
collections required by `shorty`.

```rust
use shorty::setup_db;

#[tokio::main]
async fn main() -> Result<(), String> {
    setup_db().await?;
}
```

## Shortening URLs

Now that the database is ready to go, you are ready to shorten the URLs of your
choice. Call the following function to shorten a URL.

```rust
use shorty::create_url;

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let short_id = create_url("https://example.com").await?;
    println!("Shortened URL to ID: {short_id}");
    Ok(())
}
```

## Lengthening URLs

Once shortened, you can fetch the full URL from the short ID. If the short ID
was found, the function will increment the view count and return the full URL.

```rust
use shorty::get_url;

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let full_url = get_url("test_url_id").await?;

    // `full_url` will be `None` if the URL was not found.
    let full_url = full_url.unwrap();
    println!("Full URL is: {full_url}");
    Ok(())
}
```
