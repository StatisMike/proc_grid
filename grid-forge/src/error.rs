use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct BuilderError {
    missing_fields: Vec<&'static str>,
}

impl BuilderError {
    pub(crate) fn new() -> Self {
        Self {
            missing_fields: Vec::new(),
        }
    }

    pub(crate) fn add_missing_field(&mut self, field: &'static str) {
        self.missing_fields.push(field);
    }

    pub(crate) fn try_throw(self) -> Result<(), Self> {
        if self.missing_fields.is_empty() {
            return Ok(());
        }

        Err(self)
    }
}

impl Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "there are missing field values: {missing}",
            missing = self.missing_fields.join(", ")
        )
    }
}

impl Error for BuilderError {}
