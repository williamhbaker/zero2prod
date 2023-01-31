use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirm_subscription_with_valid_token() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let res = app.post_subscriptions(body.to_string()).await;
    assert!(res.status().is_success());

    let saved = sqlx::query!("SELECT status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.status, "pending_confirmation");

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let res = app
        .get_subscription_confirm(&body.get("HtmlBody").unwrap().as_str().unwrap())
        .await;
    assert_eq!(res.status(), 200);

    let saved = sqlx::query!("SELECT status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.status, "confirmed");
}
