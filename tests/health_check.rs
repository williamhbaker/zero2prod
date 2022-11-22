use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app();
    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/health", addr))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}

#[tokio::test]
async fn subscriptions_returns_200_for_valid_data() {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let addr = spawn_app();

    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{}/subscriptions", addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(res.status().is_success());
}

#[tokio::test]
async fn subscriptions_returns_400_for_invalid_data() {
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    let addr = spawn_app();

    let client = reqwest::Client::new();

    for (invalid_body, error_message) in test_cases {
        let res = client
            .post(format!("http://{}/subscriptions", addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            res.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("localhost:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("localhost:{}", port)
}
