use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;

pub fn run(listener: TcpListener, db: PgPool) -> Result<Server, std::io::Error> {
    let db = web::Data::new(db);

    Ok(HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db.clone())
    })
    .listen(listener)?
    .run())
}
