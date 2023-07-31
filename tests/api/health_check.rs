use crate::helpers::spawn_app;
use crate::helpers::TestApp;

#[actix_web::test]
async fn health_check_works() {
    let testapp: TestApp = spawn_app().await;
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(&format!("{}/health_check", &testapp.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
