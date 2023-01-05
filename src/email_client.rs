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
    ) -> Self {
        EmailClient {
            http_client: Client::new(),
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
            from: self.sender.as_ref().to_string(),
            to: recipient.as_ref().to_string(),
            subject: subject.to_string(),
            html_body: html_body.to_string(),
            text_body: text_body.to_string(),
        };

        self.http_client
            .post(&url)
            .json(&request_body)
            .header("Authorization", self.authorization_token.expose_secret())
            .send()
            .await?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    #[tokio::test]
    async fn send_email_sends_email() {
        let server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(sender, server.uri(), Secret::new(Faker.fake()));

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let recipient = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(recipient, &subject, &content, &content)
            .await;
    }
}
