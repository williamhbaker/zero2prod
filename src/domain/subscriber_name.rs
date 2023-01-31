use unicode_segmentation::UnicodeSegmentation;

#[derive(thiserror::Error, Debug)]
pub enum NameError {
    #[error("name must not be empty")]
    EmptyOrWhitespace,
    #[error("name must be less than 256 characters, was {} characters", .0.len())]
    TooLong(String),
    #[error("name {0} contains forbidden characters")]
    ForbiddenCharacters(String),
}

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, NameError> {
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        if s.graphemes(true).count() > 256 {
            Err(NameError::TooLong(s))
        } else if s.trim().is_empty() {
            Err(NameError::EmptyOrWhitespace)
        } else if s.chars().any(|c| forbidden_characters.contains(&c)) {
            Err(NameError::ForbiddenCharacters(s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};

    use super::SubscriberName;

    #[test]
    fn whitespace_name_is_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn long_name_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn valid_name() {
        let name = "Some Guy".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
