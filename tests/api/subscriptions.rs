use crate::helpers::spawn_app;
use crate::helpers::TestApp;
use random_string::generate;
use urlencoding::decode;

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let testapp: TestApp = spawn_app().await;
    let pg_pool = testapp.db_pool;
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let client_name = format!("monster%20{}", generate(20, charset));
    let client_email = format!("{}@{}.com", generate(5, charset), generate(5, charset));
    let client = reqwest::Client::new();
    let body = format!("name={}&email={}", &client_name, &client_email);

    //Act
    let response = client
        .post(&format!("{}/subscriptions", &testapp.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    //Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email,name FROM subscriptions",)
        .fetch_one(&pg_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, client_email);
    assert_eq!(saved.name, decode(&client_name).unwrap());
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let testapp: TestApp = spawn_app().await;
    let client = reqwest::Client::new();
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
        let response = client
            .post(&format!("{}/subscriptions", &testapp.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
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
    let client = reqwest::Client::new();
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let client_name: String = format!("name=monster%20{}", generate(10, charset));
    let client_email: String = format!(
        "email={}%40{}.com",
        generate(5, charset),
        generate(5, charset)
    );
    let test_cases = vec![
        (format!("name=&email={}", client_email), "empty name"),
        (format!("name={}&email=", client_name), "empty name"),
        (
            format!("name={}&email=not-an-email", client_name),
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        //Act
        let response = client
            .post(&format!("{}/subscriptions", &testapp.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
