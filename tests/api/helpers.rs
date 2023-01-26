use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("http://{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_subscription_confirm(&self, token: &str) -> reqwest::Response {
        reqwest::Client::new()
            .get(format!("http://{}/subscriptions/confirm", self.address))
            .query(&[("token", token)])
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let mut config = get_configuration().expect("Failed to load configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    config.application.port = 0;
    config.email.base_url = email_server.uri();
    configure_db(&config.database).await;

    let app = Application::build(config.clone())
        .await
        .expect("Failed to build app");

    let db_pool = get_connection_pool(&config).await;

    let address = format!("localhost:{}", app.port());
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address,
        db_pool,
        email_server,
    }
}

async fn configure_db(config: &DatabaseSettings) {
    let mut connection = PgConnection::connect(&config.dsn_without_db().expose_secret())
        .await
        .expect("Failed to connect to database");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let dsn = config.dsn();

    let connection_pool = PgPool::connect(&dsn.expose_secret())
        .await
        .expect("Failed to create connection pool");
    sqlx::migrate!("./dbinit/postgres")
        .run(&connection_pool)
        .await
        .expect("Failed to apply migrations to temporary database");
}
