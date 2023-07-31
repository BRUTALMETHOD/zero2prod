use crate::helpers::spawn_app;
use crate::helpers::TestApp;
use random_string::generate;
use urlencoding::decode;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let testapp: TestApp = spawn_app().await;
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let client_name = format!("monster%20{}", generate(20, charset));
    let client_email = format!("{}@{}.com", generate(5, charset), generate(5, charset));
    let body = format!("name={}&email={}", &client_name, &client_email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&testapp.email_server)
        .await;
    //Act
    let response = testapp.post_subscriptions(body.into()).await;
    //Assert
    assert_eq!(200, response.status().as_u16());
}

#[actix_web::test]
async fn subscribe_persists_the_new_subscriber() {
    //Arrange
    let app = spawn_app().await;
    let body = "name=le%20name&email=le_email%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    //Act
    app.post_subscriptions(body.into()).await;
    //Assert
    let saved = sqlx::query!("SELECT email,name,status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, "le_email@gmail.com");
    assert_eq!(saved.name, "le name");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let testapp: TestApp = spawn_app().await;
    //let client = reqwest::Client::new();
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let client_name: String = format!("name=monster%20{}", generate(10, charset));
    let client_email: String = format!(
        "email={}%40{}.com",
        generate(5, charset),
        generate(5, charset)
    );
    let test_cases = vec![
        (client_email, "missing the email"),
        (client_name, "missing the name"),
        (String::from(""), "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        //Act
        let response = testapp.post_subscriptions(invalid_body.into()).await;
        //Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "the API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[actix_web::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    //Arrange
    let testapp: TestApp = spawn_app().await;
    //let client = reqwest::Client::new();
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let client_name: String = format!("name=monster%20{}", generate(10, charset));
    let client_email: String = format!(
        "email={}%40{}.com",
        generate(5, charset),
        generate(5, charset)
    );
    let test_cases = vec![
        (format!("name=&email={}", client_email), "empty name"),
        (format!("name={}&email=", client_name), "empty email"),
        (
            format!("name={}&email=not-an-email", client_name),
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        //Act
        let response = testapp.post_subscriptions(body.into()).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[actix_web::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let client_name = format!("monster%20{}", generate(20, charset));
    let client_email = format!("{}@{}.com", generate(5, charset), generate(5, charset));
    let body = format!("name={}&email={}", &client_name, &client_email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
}

#[actix_web::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let client_name = format!("monster%20{}", generate(20, charset));
    let client_email = format!("{}@{}.com", generate(5, charset), generate(5, charset));
    let body = format!("name={}&email={}", &client_name, &client_email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    // Get the first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    assert_eq!(confirmation_links.plain_text, confirmation_links.html);
}
