use once_cell::sync::Lazy;
use random_string::generate;
use reqwest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use tokio;
use urlencoding::decode;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

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
            200,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }
}
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
// launch app in background
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Unable to bind to random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", &port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server =
        zero2prod::startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    //create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    //migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
