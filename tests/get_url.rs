mod common;

use shorty::{create_url, get_url};

#[tokio::test]
async fn test_get_url() {
    common::setup().await;

    const URL: &str = "https://example.com";
    let short_id = create_url(URL).await.expect("could not shorten URL");

    let full_url = get_url(&short_id).await;
    assert!(full_url.is_ok());

    let full_url = full_url.unwrap();
    assert!(full_url.is_some());

    let full_url = full_url.unwrap();
    assert_eq!(full_url, URL);

    // Cleanup
    common::delete_by_short_id(&short_id).await;
}

#[tokio::test]
async fn test_get_url_invalid_id() {
    common::setup().await;

    let full_url = get_url("this_id_does_not_exist").await;
    assert!(full_url.is_ok());

    let full_url = full_url.unwrap();
    assert!(full_url.is_none());
}
