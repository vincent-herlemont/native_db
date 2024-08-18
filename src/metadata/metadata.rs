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
    pub(crate) fn new(previous_version: String, previous_native_model_version: String) -> Self {
        let current_version = Version::parse(CURRENT_VERSION).unwrap();
        let current_native_model_version = Version::parse(CURRENT_NATIVE_MODEL_VERSION).unwrap();

        Self {
            current_version: current_version.to_string(),
            current_native_model_version: current_native_model_version.to_string(),
            previous_version: Some(previous_version.to_string()),
            previous_native_model_version: Some(previous_native_model_version.to_string()),
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
