use std::{borrow::Cow, fmt::Display};

pub trait Module {
    fn module_name<'a>(&self) -> ModuleName<'a>;

    fn setup(&self) -> Result<(), ()>;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleName<'a>(Cow<'a, str>);

impl ModuleName<'_> {
    pub const STORAGE: Self = Self(Cow::Borrowed("storage"));
    // const STORAGE: Self = Self(Cow::Borrowed("storage"));
}

impl From<&'static str> for ModuleName<'static> {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl Display for ModuleName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
