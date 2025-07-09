use serde::{Deserialize, Serialize};

// Import v0.8.1 version as native_db for macro expansion
use native_db_v0_8_x as native_db;
use native_db_v0_8_x::ToKey;

// Import native_model macro version matched with the v0.8.1 version native_db for macro expansion.
use native_model_v0_4_x as native_model;
use native_model_v0_4_x::{native_model, Model};

// Model for v0.8.x
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db_v0_8_x::native_db]
pub struct V08xModel {
    #[primary_key]
    pub id: u32,
    pub name: String,
}
