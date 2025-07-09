# Native DB Major Version Upgrade Example

This example demonstrates how to upgrade a Native DB database from an older major version (v0.8.x) to the current version, handling breaking changes between incompatible versions.

## Overview

When Native DB releases a new major version with breaking changes, databases created with older versions cannot be directly opened. This example shows how to:

1. Define models for both the old and new versions
2. Use the [`upgrade`](https://docs.rs/native_db/latest/native_db/struct.Builder.html#method.upgrade) method to migrate data
3. Handle version conversion between different model versions
4. Preserve data integrity during the migration

## Project Structure

```
major_upgrade/
├── Cargo.toml              # Dependencies for both versions
├── src/
│   ├── lib.rs             # Module exports
│   ├── main_old.rs        # Creates v0.8.x database
│   ├── new_main.rs        # Handles upgrade to current version
│   └── models/
│       ├── mod.rs         # Model exports
│       ├── v08x.rs        # v0.8.x model definition
│       └── current_version.rs  # Current version model
└── tests/
    └── test_main.rs       # Integration tests
```

## Key Components

### 1. Dependency Configuration (Cargo.toml)

The project uses different package names to import multiple versions of Native DB:

```toml
# Current version (requires >=0.9.x for major upgrade functionality)
native_model_current = { package = "native_model", version = "0.6.2" }
native_db_current = { package = "native_db", version = "0.9.0" }

# Previous version (from crates.io)
native_model_v0_4_x = { package = "native_model", version = "0.4.20" }
native_db_v0_8_x = { package = "native_db", version = "0.8.2" }
```

### 2. Model Definitions

#### Old Version Model (v08x.rs)

```rust
// Import v0.8.x versions with aliases
use native_db_v0_8_x as native_db;
use native_model_v0_4_x as native_model;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db_v0_8_x::native_db]
pub struct V08xModel {
    #[primary_key]
    pub id: u32,
    pub name: String,
}
```

#### Current Version Model (current_version.rs)

```rust
// Import current versions with aliases
use native_db_current as native_db;
use native_model_current as native_model;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db_current::native_db]
pub struct CurrentModel {
    #[primary_key]
    pub id: u32,
    pub name: String,
}

// Conversion from old to new
impl From<crate::models::v08x::V08xModel> for CurrentModel {
    fn from(v08x_model: crate::models::v08x::V08xModel) -> Self {
        Self {
            id: v08x_model.id,
            name: v08x_model.name,
        }
    }
}
```

> **Note on Native Model Version Numbers**: In this example, both the old and new models use `version = 1` for the `native_model` attribute. You can choose to either reset the version number to 1 for the new model (as shown here) or continue incrementing from the previous version. The choice depends on whether your `native_model` is used by external systems (network communication, file formats, etc.) - if so, you should maintain version continuity to ensure compatibility. If the model is only used internally within your database, you can reset the version number.

### 3. Upgrade Process (new_main.rs)

The upgrade process uses the [`Builder::upgrade`](https://docs.rs/native_db/latest/native_db/struct.Builder.html#method.upgrade) method with simplified error handling:

```rust
use native_db_current::upgrade::UpgradeResultExt;

let upgraded_db = CurrentBuilder::new().upgrade(&current_models, &db_path, |new_txn| {
    // 1. Open the old database
    let mut old_models = V08xModels::new();
    old_models.define::<V08xModel>()
        .upgrade_context("defining old model")?;
    
    let old_db = V08xBuilder::new()
        .open(&old_models, &db_path)
        .upgrade_context("opening old database")?;

    // 2. Read all data from old database
    let old_txn = old_db.r_transaction()
        .upgrade_context("creating read transaction")?;
    let scan = old_txn.scan().primary()
        .upgrade_context("creating primary scan")?;

    // 3. Migrate each item
    for item_result in scan.all()? {
        let old_item: V08xModel = item_result
            .upgrade_context("reading item from old database")?;
        let new_item: CurrentModel = old_item.into(); // Conversion
        new_txn.insert(new_item)?;
    }

    Ok(())
})?;
```

#### Error Handling with UpgradeResultExt

The `UpgradeResultExt` trait simplifies error handling when working with different database versions:

```rust
// Import the extension trait
use native_db_current::upgrade::UpgradeResultExt;

// Use .upgrade_context() to convert any error with context
old_db.r_transaction()
    .upgrade_context("creating read transaction")?;

// For items in loops, use .upgrade_with_item() to include debug info
item_result
    .upgrade_with_item("processing", &item)?;
```

This converts incompatible error types between versions and adds helpful context for debugging.
