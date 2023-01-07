use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: Client,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        sender: SubscriberEmail,
        base_url: String,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        EmailClient {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            sender,
            base_url,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject,
            html_body: html_body,
            text_body: text_body,
        };

        self.http_client
            .post(&url)
            .json(&request_body)
            .header("Authorization", self.authorization_token.expose_secret())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, Request, ResponseTemplate,
    };

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            match request.body_json::<serde_json::Value>() {
                Ok(body) => {
                    body.get("From").is_some()
                        && body.get("To").is_some()
                        && body.get("Subject").is_some()
                        && body.get("HtmlBody").is_some()
                        && body.get("TextBody").is_some()
                }
                Err(_) => false,
            }
        }
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(uri: String) -> EmailClient {
        EmailClient::new(
            email(),
            uri,
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(10),
        )
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    #[tokio::test]
    async fn send_email_sends_expected_request() {
        let server = MockServer::start().await;
        let email_client = email_client(server.uri());

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let res = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(res);
    }

    #[tokio::test]
    async fn send_email_errors_on_server_error() {
        let server = MockServer::start().await;
        let email_client = email_client(server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let res = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(res);
    }

    #[tokio::test]
    async fn send_email_times_out() {
        let server = MockServer::start().await;
        let email_client = email_client(server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&server)
            .await;

        let res = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(res);
    }
}
