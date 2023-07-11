use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration, email_client::EmailClient, startup::run,
    telemetry::get_subscriber, telemetry::init_subscriber,
};

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // logging subscriber setup
    let subscriber = get_subscriber("zero2prod".into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    // configuration setup
    let configuration = get_configuration().expect("Failed to read configuration.");

    // db setup
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // email client
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(configuration.email_client.base_url, sender_email);
    // startup
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::info!(
        "Application listening on: {}:{}",
        configuration.application.host,
        configuration.application.port
    );
    let listener = TcpListener::bind(&address).expect("Failed to bind to port.");
    run(listener, connection_pool, email_client)?.await
}
