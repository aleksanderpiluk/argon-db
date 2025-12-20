use std::{borrow::Cow, fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KVTableName<'a>(Cow<'a, str>);

impl Display for KVTableName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_ref())
    }
}

impl KVTableName<'_> {
    pub fn to_owned(&self) -> KVTableName<'static> {
        let s = self.0.to_string();

        KVTableName(Cow::Owned(s))
    }
}

impl KVTableName<'static> {
    pub const unsafe fn from_str_unchecked(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl FromStr for KVTableName<'_> {
    type Err = KVTableNameConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_string();

        if s.len() == 0 || s.len() > 64 {
            return Err(KVTableNameConversionError::new(s));
        }

        for c in s.chars() {
            let is_valid_char = c.is_ascii_lowercase() || c == '_';

            if !is_valid_char {
                return Err(KVTableNameConversionError::new(s));
            }
        }

        Ok(Self(Cow::Owned(s)))
    }
}

#[derive(Debug)]
pub struct KVTableNameConversionError {
    pub given_value: String,
}

impl KVTableNameConversionError {
    pub fn new(s: String) -> Self {
        Self { given_value: s }
    }
}

impl std::fmt::Display for KVTableNameConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for KVTableNameConversionError {}
