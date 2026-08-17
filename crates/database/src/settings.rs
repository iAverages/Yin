//! Validated values shared by settings writers and repositories.
//!
//! Constructors validate and normalize input before it can reach an upsert.
//! Keep fields private and do not add unchecked constructors, mutable string
//! access, or derived deserialization that could bypass validation.

use std::fmt;

/// A trimmed, non-empty command prefix of at most 16 characters, without whitespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPrefix(String);

impl CommandPrefix {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.is_empty() || value.chars().count() > 16 || value.chars().any(char::is_whitespace)
        {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A normalized translation language using the supported language/subtag syntax.
///
/// This checks syntax, not membership in an ISO language registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationLanguage(String);

impl TranslationLanguage {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim().to_ascii_lowercase().replace('_', "-");
        let value = match value.as_str() {
            "zh" | "cn" | "zh-hans" => "zh-cn",
            "tw" | "hk" | "zh-hk" | "zh-mo" | "zh-hant" => "zh-tw",
            "jp" => "ja",
            "kr" => "ko",
            "ua" => "uk",
            value => value,
        };
        let mut parts = value.split('-');
        let primary = parts.next()?;
        let subtag = parts.next();
        if parts.next().is_some()
            || !(2..=3).contains(&primary.len())
            || !primary.bytes().all(|byte| byte.is_ascii_alphabetic())
            || subtag.is_some_and(|subtag| {
                !(2..=4).contains(&subtag.len())
                    || !subtag.bytes().all(|byte| byte.is_ascii_alphabetic())
            })
        {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

/// A lowercase command name of 1-32 ASCII letters, digits, underscores or hyphens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommandName(String);

impl CustomCommandName {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim().to_ascii_lowercase();
        if value.is_empty()
            || value.len() > 32
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CustomCommandName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Trimmed custom-command text containing 1-2,000 characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommandText(String);

impl CustomCommandText {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.is_empty() || value.chars().count() > 2000 {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_are_trimmed_and_bounded_by_characters() {
        assert_eq!(CommandPrefix::parse(" ! ").unwrap().as_str(), "!");
        assert!(CommandPrefix::parse(&"界".repeat(16)).is_some());
        assert!(CommandPrefix::parse(&"界".repeat(17)).is_none());
        for invalid in ["", "  ", "a b", "a\tb", "a\nb", "a\u{2003}b"] {
            assert!(CommandPrefix::parse(invalid).is_none(), "{invalid:?}");
        }
    }

    #[test]
    fn translation_languages_preserve_normalization_and_aliases() {
        for (input, expected) in [
            (" en ", "en"),
            ("fil", "fil"),
            ("PT_BR", "pt-br"),
            ("zh", "zh-cn"),
            ("cn", "zh-cn"),
            ("zh-Hans", "zh-cn"),
            ("tw", "zh-tw"),
            ("hk", "zh-tw"),
            ("zh-HK", "zh-tw"),
            ("zh-MO", "zh-tw"),
            ("zh-Hant", "zh-tw"),
            ("jp", "ja"),
            ("kr", "ko"),
            ("ua", "uk"),
        ] {
            let language = TranslationLanguage::parse(input).unwrap();
            assert_eq!(language.as_str(), expected);
            assert_eq!(language.into_string(), expected);
        }
    }

    #[test]
    fn malformed_translation_languages_are_rejected() {
        for invalid in [
            "",
            " ",
            "e",
            "unknown",
            "e1",
            "éé",
            "en-",
            "en-u",
            "en-123",
            "en-abcde",
            "en-us-extra",
            "en us",
            "en--us",
            "en/ja",
        ] {
            assert!(TranslationLanguage::parse(invalid).is_none(), "{invalid:?}");
        }
    }

    #[test]
    fn command_names_are_canonical_and_bounded() {
        assert_eq!(
            CustomCommandName::parse(" Road-Map_1 ").unwrap().as_str(),
            "road-map_1"
        );
        assert!(CustomCommandName::parse(&"a".repeat(32)).is_some());
        assert!(CustomCommandName::parse(&"a".repeat(33)).is_none());
        for invalid in ["", " ", "road map", "a/b", "é", "a.b"] {
            assert!(CustomCommandName::parse(invalid).is_none(), "{invalid:?}");
        }
    }

    #[test]
    fn command_text_is_trimmed_and_bounded_by_characters() {
        assert_eq!(
            CustomCommandText::parse(" Hello\nworld! ")
                .unwrap()
                .as_str(),
            "Hello\nworld!"
        );
        assert!(CustomCommandText::parse(&"界".repeat(2000)).is_some());
        assert!(CustomCommandText::parse(&"界".repeat(2001)).is_none());
        assert!(CustomCommandText::parse("").is_none());
        assert!(CustomCommandText::parse(" \n\t ").is_none());
    }
}
