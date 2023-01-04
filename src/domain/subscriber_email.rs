use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("invalid email {}", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claim::assert_err;
    use fake::{faker::internet::en::SafeEmail, Fake};

    use super::SubscriberEmail;

    #[test]
    fn invalid_email_rejected() {
        let email = "hello there".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            quickcheck::empty_shrinker()
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_email_ok(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
