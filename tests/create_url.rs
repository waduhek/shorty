mod common;

use shorty::create_url;

#[tokio::test]
async fn test_create_new_url() {
    common::setup().await;

    const URL: &str = "https://example.com";
    let create_url_result = create_url(URL).await;
    assert!(
        create_url_result.is_ok(),
        "received an error while creating url: {}",
        create_url_result.err().unwrap()
    );

    common::delete_by_short_id(&create_url_result.unwrap()).await;
}
