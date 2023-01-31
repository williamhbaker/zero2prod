use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct Parameters {
    token: String,
}

#[tracing::instrument(
    name = "Confirming subscriber",
    skip(db_pool, token),
    fields(
        token=%token.token,
    ),
)]
pub async fn confirm_subscription(
    db_pool: web::Data<PgPool>,
    token: web::Query<Parameters>,
) -> HttpResponse {
    let t = token.into_inner();

    match sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id IN (
            SELECT subscriber_id
            FROM subscription_tokens
            WHERE subscription_token = $1
        )
        RETURNING id;
        "#,
        t.token,
    )
    .fetch_one(db_pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => match e {
            sqlx::Error::RowNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}
