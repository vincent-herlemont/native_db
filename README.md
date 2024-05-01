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
native_db = "0.6.0"
native_model = "0.4.14"
```

NOTE: `native_db` requires `native_model` to work.

# Status

Active development. The API is not stable yet and may change in the future.

# Usage

- With Tauri: [native_db_tauri_vanilla](https://github.com/vincent-herlemont/native_db_tauri_vanilla)

# Usual API
- [**DatabaseBuilder**](https://docs.rs/native_db/latest/native_db/struct.DatabaseBuilder.html)  
    - [**define**](https://docs.rs/native_db/latest/native_db/struct.DatabaseBuilder.html#method.define) a model.
    - [**create**](https://docs.rs/native_db/latest/native_db/struct.DatabaseBuilder.html#method.create) / [**open**](https://docs.rs/native_db/latest/native_db/struct.DatabaseBuilder.html#method.open) a database.
    - [**create_in_memory**](https://docs.rs/native_db/latest/native_db/struct.DatabaseBuilder.html#method.create_in_memory) an in-memory database.
- [**Database**](https://docs.rs/native_db/latest/native_db/struct.Database.html)
    - [**snapshot**](https://docs.rs/native_db/latest/native_db/struct.Database.html#method.snapshot) the database.
    - **rw_transaction** open a read-write transaction.
        - [**insert**](https://docs.rs/native_db/latest/native_db/transaction/struct.RwTransaction.html#method.insert) a new item.
        - [**update**](https://docs.rs/native_db/latest/native_db/transaction/struct.RwTransaction.html#method.update) an existing item.
        - [**remove**](https://docs.rs/native_db/latest/native_db/transaction/struct.RwTransaction.html#method.remove) an existing item.
        - [**commit**](https://docs.rs/native_db/latest/native_db/transaction/struct.RwTransaction.html#method.commit) the transaction.
        - [**migrate**](https://docs.rs/native_db/latest/native_db/transaction/struct.RwTransaction.html#method.migrate) a model.
        - plus all read-only transaction APIs.
    - **r_transaction** open a read-only transaction.
        - **get**
            - [**primary**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.RGet.html#method.primary) an item by its primary key.
            - [**secondary**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.RGet.html#method.secondary) an item by its secondary key.
        - **scan**
            - **primary**
                - [**all**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.PrimaryScan.html#method.all) items.
                - [**start_with**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.PrimaryScan.html#method.start_with) items with a primary key starting with a given value.
                - [**range**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.PrimaryScan.html#method.range) items with a primary key in a given range.
            - **secondary**
                - [**all**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.SecondaryScan.html#method.all) items with a given secondary key.
                - [**start_with**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.SecondaryScan.html#method.start_with) items with a secondary key starting with a given value.
                - [**range**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.SecondaryScan.html#method.range) items with a secondary key in a given range.
        - **len**
            - [**primary**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.RLen.html#method.primary) the number of items.
            - [**secondary**](https://docs.rs/native_db/latest/native_db/transaction/query/struct.RLen.html#method.secondary) the number of items with a given secondary key.
    - **watch** real-time subscriptions via [std channel](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html) based or [tokio channel](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html) based depending on the feature `tokio`.
        - **get**
            - [**primary**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchGet.html#method.primary) an item by its primary key.
            - [**secondary**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchGet.html#method.secondary) an item by its secondary key.
        - **scan**
            - **primary**
                - [**all**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchScanPrimary.html#method.all) items.
                - [**start_with**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchScanPrimary.html#method.start_with) items with a primary key starting with a given value.
                - [**range**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchScanPrimary.html#method.range) items with a primary key in a given range.
            - **secondary**
                - [**all**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchScanSecondary.html#method.all) items with a given secondary key.
                - [**start_with**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchScanSecondary.html#method.start_with) items with a secondary key starting with a given value.
                - [**range**](https://docs.rs/native_db/latest/native_db/watch/query/struct.WatchScanSecondary.html#method.range) items with a secondary key in a given range.


# Example

```rust
use serde::{Deserialize, Serialize};
use native_db::*;
use native_model::{native_model, Model};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String,
}

fn main() -> Result<(), db_type::Error> {
    let mut builder = DatabaseBuilder::new();
    // Initialize the model
    builder.define::<Item>()?;
    
    // Create a database in memory
    let mut db = builder.create_in_memory()?;
    
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