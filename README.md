# Native DB

[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_linux.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_linux.yml)
[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_macos.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_macos.yml)
[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_windows.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_windows.yml)
[![](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_ios.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_ios.yml)
[![)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_android.yml/badge.svg)](https://github.com/vincent-herlemont/native_db/actions/workflows/build_test_android.yml)


[![Crates.io](https://img.shields.io/crates/v/native_db)](https://crates.io/crates/native_db)
[![Documentation](https://docs.rs/native_db/badge.svg)](https://docs.rs/native_db)
[![License](https://img.shields.io/crates/l/native_db)](LICENSE)

<!-- ALL-CONTRIBUTORS-BADGE:START - Do not remove or modify this section -->
[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg)](#contributors-)
<!-- ALL-CONTRIBUTORS-BADGE:END -->


Here's a drop-in, fast, embedded database for multi-platform apps (server, desktop, mobile). Sync Rust types effortlessly. Enjoy! 😌🍃.

# Features

- Simple API 🦀.
- Support for **multiple indexes** (primary, secondary, unique, non-unique, optional).
- Minimal boilerplate see [benchmarks](./benches).
- Transparent serialization/deserialization using [native_model](https://github.com/vincent-herlemont/native_model).
- **Automatic model migration** 🌟.
- **Thread-safe** and fully **ACID-compliant** transactions provided by [redb](https://github.com/cberner/redb).
- **Real-time** subscription with filters for `insert`, `update` and `delete` operations.
- Compatible with all Rust types (`enum`, `struct`, `tuple` etc.).
- **Hot snapshots**.

# Installation

Add this to your `Cargo.toml`:
```toml
[dependencies]
native_db = "0.7.1"
native_model = "0.4.19"
```

NOTE: `native_db` requires `native_model` to work.

# Status

Active development. The API is not stable yet and may change in the future.

# How to use?

- [Documentation API](https://docs.rs/native_db/latest/native_db/#api)
- [Quick Start](https://docs.rs/native_db/latest/native_db/#quick_start)
- Full example with Tauri: [native_db_tauri_vanilla](https://github.com/vincent-herlemont/native_db_tauri_vanilla)

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
    let rw = db.rw_transaction().unwrap();
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
    for item in r.scan().secondary::<Item>(ItemKey::name)?.start_with("red") {
        println!("data name=\"red\": {:?}", item);
    }
    
    // Remove data (open a read-write transaction)
    let rw = db.rw_transaction()?;
    rw.remove(retrieve_data)?;
    rw.commit()?;
    Ok(())
}
```

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/elliot14A"><img src="https://avatars.githubusercontent.com/u/84667163?v=4?s=100" width="100px;" alt="Akshith Madhur"/><br /><sub><b>Akshith Madhur</b></sub></a><br /><a href="https://github.com/vincent-herlemont/native_db/commits?author=elliot14A" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->