//! CLI tool for shortening and lengthening URLs using the `shorty` library.
//!
//! # Pre-requisites
//!
//! The tool requires the environment variables listed in
//! [`sample.env`](sample.env) to be present.
//!
//! # Usage
//!
//! The tool provides 2 commands "shorten" and "lengthen".
//!
//! ## Shortening URLs
//!
//! The command format for shortening URLs is:
//!
//! ```bash
//! $ cargo run -- shorten <URL>
//! ```
//!
//! For example, run the following command to shorten "https://example.com":
//!
//! ```bash
//! $ cargo run -- shorten https://example.com
//! ```
//!
//! After successfully shortening the URL, the short ID will be printed as the
//! output.
//!
//! ## Lengthening URLs
//!
//! The command format for lengthening URLs is:
//!
//! ```bash
//! $ cargo run -- lengthen <short_id>
//! ```
//!
//! For example, run the following command to lengthen "abcdAB123" to the full
//! URL:
//!
//! ```
//! $ cargo run -- lengthen abcdAB123
//! ```
//!
//! After successfully lengthening the URL, the full UR will be printed as the
//! output. If the provided short ID was not found, "not found" will be printed
//! as the output to STDERR.

#[macro_use]
extern crate lazy_static;

mod cli_utils;

use std::{env, process};

use crate::cli_utils::{ShortyArgs, ShortyCommand};

async fn handle_shorten_url(full_url: String) {
    let short_id = match shorty::create_url(&full_url).await {
        Ok(id) => id,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    println!("{short_id}");
}

async fn handle_lengthen_short_id(short_id: String) {
    let full_url = match shorty::get_url(&short_id).await {
        Ok(url) => url,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    if full_url.is_none() {
        eprintln!("not found");
        process::exit(1);
    }

    let full_url = full_url.unwrap();
    println!("{full_url}");
}

#[tokio::main]
async fn main() {
    match shorty::setup_db().await {
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
        Ok(_) => (),
    };

    let args = match ShortyArgs::build(env::args()) {
        Ok(arg) => arg,
        Err(err_string) => {
            eprintln!("{err_string}");
            process::exit(1);
        }
    };

    match args.command {
        ShortyCommand::Shorten(full_url) => handle_shorten_url(full_url).await,
        ShortyCommand::Lengthen(short_id) => {
            handle_lengthen_short_id(short_id).await
        }
    };
}
