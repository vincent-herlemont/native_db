use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub struct UpgradeRequiredError {
    pub details: Vec<String>,
    pub native_db_version: Option<(String, String)>, // (current, required)
    pub native_model_version: Option<(String, String)>, // (current, required)
    pub redb_version: Option<u8>,
}

impl fmt::Display for UpgradeRequiredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database upgrade required:")?;
        for detail in &self.details {
            write!(f, "\n{detail}")?;
        }
        Ok(())
    }
}

impl Default for UpgradeRequiredError {
    fn default() -> Self {
        Self::new()
    }
}

impl UpgradeRequiredError {
    pub fn new() -> Self {
        Self {
            details: Vec::new(),
            native_db_version: None,
            native_model_version: None,
            redb_version: None,
        }
    }

    pub fn with_native_db_version(mut self, current: String, required: String) -> Self {
        self.native_db_version = Some((current.clone(), required.clone()));
        self.details
            .push(format!("  - Native DB: {current} → {required}"));
        self
    }

    pub fn with_native_model_version(mut self, current: String, required: String) -> Self {
        self.native_model_version = Some((current.clone(), required.clone()));
        self.details
            .push(format!("  - Native Model: {current} → {required}"));
        self
    }

    pub fn with_redb_version(mut self, version: u8) -> Self {
        self.redb_version = Some(version);
        self.details
            .push(format!("  - redb format: v{version} → v2"));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.details.is_empty()
    }

    pub fn build(self) -> Result<(), Box<Self>> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(Box::new(self))
        }
    }
}
