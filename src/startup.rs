use std::net::TcpListener;

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use actix_web::{dev::Server, web, App, HttpServer};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let address = format!("{}:{}", config.application.host, config.application.port);

        let db_pool = get_connection_pool(&config).await;

        let sender = config.email.sender().expect("Invalid sender email address");
        let timeout = config.email.timeout();
        let email_client = EmailClient::new(
            sender,
            config.email.base_url,
            config.email.authorization_token,
            timeout,
        );

        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_pool(config: &Settings) -> PgPool {
    let dsn = config.database.dsn();

    PgPool::connect(&dsn.expose_secret())
        .await
        .expect("Failed to connect to database")
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run())
}
