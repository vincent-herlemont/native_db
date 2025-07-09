use super::CURRENT_NATIVE_MODEL_VERSION;
use super::CURRENT_VERSION;
use semver::Version;

pub struct Metadata {
    current_version: String,
    current_native_model_version: String,
    previous_version: Option<String>,
    previous_native_model_version: Option<String>,
}

impl Metadata {
    pub(crate) fn from_stored(stored_version: String, stored_native_model_version: String) -> Self {
        Self {
            current_version: stored_version,
            current_native_model_version: stored_native_model_version,
            previous_version: None,
            previous_native_model_version: None,
        }
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    pub fn current_native_model_version(&self) -> &str {
        &self.current_native_model_version
    }

    pub fn previous_version(&self) -> Option<&str> {
        self.previous_version.as_deref()
    }

    pub fn previous_native_model_version(&self) -> Option<&str> {
        self.previous_native_model_version.as_deref()
    }
}

impl Default for Metadata {
    fn default() -> Self {
        let current_version = Version::parse(CURRENT_VERSION).unwrap();
        let current_native_model_version = Version::parse(CURRENT_NATIVE_MODEL_VERSION).unwrap();

        Self {
            current_version: current_version.to_string(),
            current_native_model_version: current_native_model_version.to_string(),
            previous_version: None,
            previous_native_model_version: None,
        }
    }
}
