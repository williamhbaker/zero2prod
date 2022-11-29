use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to load configuration");

    let address = format!("localhost:{}", config.application_port);
    let dsn = config.database.dsn();

    let db_pool = PgPool::connect(&dsn)
        .await
        .expect("Failed to connect to database");

    let listener = TcpListener::bind(&address)?;
    run(listener, db_pool)?.await
}
