# Native DB

[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_linux.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_linux.yml)
[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_macos.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_macos.yml)
[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_windows.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_windows.yml)
[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_ios.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_ios.yml)
[![)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_android.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_android.yml)


[![Crates.io](https://img.shields.io/crates/v/native_db)](https://crates.io/crates/native_db)
[![Documentation](https://docs.rs/native_db/badge.svg)](https://docs.rs/native_db)
[![License](https://img.shields.io/crates/l/native_db)](LICENSE)

Here's a drop-in, fast, embedded database for multi-platform apps (server, desktop, mobile). Sync Rust types effortlessly. Enjoy! 😌🍃.

# Features

- Simple API 🦀.
- Support for **multiple indexes** (primary, secondary, unique, non-unique, optional).
- Fast, see [`sqlite` vs `redb` vs `native_db`](./benches/README.md) benchmarks.
- Transparent serialization/deserialization using [native_model](https://github.com/vincent-herlemont/native_model). You can use any serialization library you want (`bincode`, `postcard`, your own etc.).
- Ensure query **type safety** to prevent unexpected results caused by selecting with an incorrect type.
- **Automatic model migration** 🌟.
- **Thread-safe** and fully **ACID-compliant** transactions provided by [redb](https://github.com/cberner/redb).
- **Real-time** subscription with filters for `insert`, `update` and `delete` operations.
- Compatible with all Rust types (`enum`, `struct`, `tuple` etc.).
- **Hot snapshots**.

# Installation

Add this to your `Cargo.toml`:
```toml
[dependencies]
native_db = "0.8.2"
native_model = "0.4.20"
```

NOTE: `native_db` requires `native_model` to work.

# Status

Active development. The API is not stable yet and may change in the future.

# How to use?

- [Documentation API](https://docs.rs/native_db/latest/native_db/#api)
- [Quick Start](https://docs.rs/native_db/latest/native_db/#quick_start)
- Full example with Tauri: [native_db_tauri_vanilla](https://github.com/vincent-herlemont/native_db_tauri_vanilla)

# Projects using Native DB

- [polly-scheduler](https://github.com/dongbin86/polly-scheduler)

If you want to propose your project or company that uses Native DB, please open a PR.

# Example

```rust
use serde::{Deserialize, Serialize};
use native_db::*;
use native_model::{native_model, Model};
use once_cell::sync::Lazy;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String,
}

// Define the models
// The lifetime of the models needs to be longer or equal to the lifetime of the database.
// In many cases, it is simpler to use a static variable but it is not mandatory.
static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    models
});

fn main() -> Result<(), db_type::Error> {
    // Create a database in memory
    let mut db = Builder::new().create_in_memory(&MODELS)?;
    
    // Insert data (open a read-write transaction)
    let rw = db.rw_transaction()?;
    rw.insert(Item { id: 1, name: "red".to_string() })?;
    rw.insert(Item { id: 2, name: "green".to_string() })?;
    rw.insert(Item { id: 3, name: "blue".to_string() })?;
    rw.commit()?;
    
    // Open a read-only transaction
    let r = db.r_transaction()?;
    // Retrieve data with id=3 
    let retrieve_data: Item = r.get().primary(3_u32)?.unwrap();
    println!("data id='3': {:?}", retrieve_data);
    // Iterate items with name starting with "red"
    for item in r.scan().secondary::<Item>(ItemKey::name)?.start_with("red")? {
        println!("data name=\"red\": {:?}", item);
    }
    
    // Remove data (open a read-write transaction)
    let rw = db.rw_transaction()?;
    rw.remove(retrieve_data)?;
    rw.commit()?;
    Ok(())
}
```
