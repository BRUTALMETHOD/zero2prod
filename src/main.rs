use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::startup::Application;
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
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
