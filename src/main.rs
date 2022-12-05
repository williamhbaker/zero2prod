use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to load configuration");

    let address = format!("localhost:{}", config.application_port);
    let dsn = config.database.dsn();

    let db_pool = PgPool::connect(&dsn)
        .await
        .expect("Failed to connect to database");

    let listener = TcpListener::bind(&address)?;
    run(listener, db_pool)?.await
}
