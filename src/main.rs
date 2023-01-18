use zero2prod::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to load configuration");
    let app = Application::build(config).await?;
    app.run_until_stopped().await?;
    Ok(())
}
