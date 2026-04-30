use std::{borrow::Cow, fmt::Display, str::FromStr};

use rand::distr::{Alphabetic, SampleString};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KVTableId<'a>(Cow<'a, str>);

impl AsRef<str> for KVTableId<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl KVTableId<'static> {
    pub const unsafe fn from_str_unchecked(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub fn new_unique() -> Self {
        let s = Alphabetic
            .sample_string(&mut rand::rng(), 20)
            .to_ascii_lowercase();

        Self(Cow::Owned(s))
    }
}

impl KVTableId<'_> {
    pub fn to_owned(&self) -> KVTableId<'static> {
        let s = self.0.to_string();

        KVTableId(Cow::Owned(s))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for KVTableId<'_> {
    type Err = KVTableIdConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_string();

        if s.len() == 0 || s.len() > 24 {
            return Err(KVTableIdConversionError::new(s));
        }

        for c in s.chars() {
            let is_valid_char = c.is_ascii_lowercase() || c == '_';

            if !is_valid_char {
                return Err(KVTableIdConversionError::new(s));
            }
        }

        Ok(Self(Cow::Owned(s)))
    }
}

#[derive(Debug)]
pub struct KVTableIdConversionError {
    pub given_value: String,
}

impl KVTableIdConversionError {
    pub fn new(s: String) -> Self {
        Self { given_value: s }
    }
}

impl Display for KVTableIdConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Given value cannot be converted to table id: {}",
            self.given_value
        )
    }
}

impl std::error::Error for KVTableIdConversionError {}
