use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration, startup::run, telemetry::get_subcriber,
    telemetry::init_subscriber,
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
    let subscriber = get_subcriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    // configuration setup
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind to port.");

    // db setup
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    // startup
    run(listener, connection_pool)?.await
}
