use serde::{Deserialize, Serialize};

// Import current version as native_db for macro expansion
use native_db_current as native_db;
use native_db_current::ToKey;

// Import native_model macro version matched with the current version native_db for macro expansion.
use native_model_current as native_model;
use native_model_current::{native_model, Model};

// Model for current version
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
// We no need to add the `from` attribute here, we manually implement
// conversion between the two models using `.into()` method.
// Maybe we could  reset the version number too. And set it to 1.
#[native_model(id = 1, version = 1)]
#[native_db_current::native_db]
pub struct CurrentModel {
    #[primary_key]
    pub id: u32,
    pub name: String,
}

// Upgrade from v0.8.x to current version
impl From<crate::models::v08x::V08xModel> for CurrentModel {
    fn from(v08x_model: crate::models::v08x::V08xModel) -> Self {
        Self {
            id: v08x_model.id,
            name: v08x_model.name,
        }
    }
}

// Downgrade from current version to v0.8.x
impl From<CurrentModel> for crate::models::v08x::V08xModel {
    fn from(current_model: CurrentModel) -> Self {
        Self {
            id: current_model.id,
            name: current_model.name,
        }
    }
}
