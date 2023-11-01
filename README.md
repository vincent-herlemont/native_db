# Important Update: This crate `struct_db` has been renamed to [`native_db`](https://crates.io/crates/native_db) to better reflect its functionality and purpose. Please update your dependencies to use [`native_db`](https://crates.io/crates/native_db) for the latest features and updates.


# Struct DB 🔧🔩 

[![Crates.io](https://img.shields.io/crates/v/struct_db)](https://crates.io/crates/struct_db)
[![Linux/Windows/macOS/Android/iOS (Build/Test/Release)](https://github.com/vincent-herlemont/struct_db/actions/workflows/build_and_test_release.yml/badge.svg)](https://github.com/vincent-herlemont/struct_db/actions/workflows/build_and_test_release.yml)
[![Documentation](https://docs.rs/struct_db/badge.svg)](https://docs.rs/struct_db)
[![License](https://img.shields.io/crates/l/struct_db)](LICENSE)

<!-- ALL-CONTRIBUTORS-BADGE:START - Do not remove or modify this section -->
[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg)](#contributors-)
<!-- ALL-CONTRIBUTORS-BADGE:END -->


Here's a drop-in, fast, embedded database solution based on [redb](https://github.com/cberner/redb) for 
multi-platform applications (server, desktop, mobile). 
It's focused on maintaining coherence between Rust types and stored data with minimal boilerplate. 
It supports multiple indexes, real-time watch with filters, schema migration. Enjoy! 😌🍃.

# Features

- Almost as fast as the storage engine [redb](https://github.com/cberner/redb).
- Embedded database (Linux, macOS, Windows, Android, iOS).
- Support multiple indexes ([unique secondary keys](https://docs.rs/struct_db/latest/struct_db/trait.ReadableTable.html#method.secondary_get)).
- Compatible with all Rust types (`enum`, `struct`, `tuple` etc.).
- [Query data](https://docs.rs/struct_db/latest/struct_db/trait.ReadableTable.html#method.primary_get) (`get`, `watch`, `iter` etc.) using explicit type or type inference. 
- [Real-time subscription](https://docs.rs/struct_db/latest/struct_db/struct.Db.html#method.primary_watch) with filters for `insert`, `update` and `delete` operations.
- [Schema migration](https://docs.rs/struct_db/latest/struct_db/struct.Tables.html#method.migrate) using native Rust coercion.
- Fully ACID-compliant transactions.
- _Add your own serialization/deserialization logic [planned*](#roadmap) (e.g: zero-copy)._
- Thread-safe.

# Status

Early development. Not ready for production. Follow the [roadmap](#roadmap) for the 1.0 release.

# How to use?

See [docs.rs](https://docs.rs/struct_db/latest/struct_db/).

# Example

```rust
use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[struct_db(
    fn_primary_key(p_key),  // required
    fn_secondary_key(s_key),  // optional
    // ... other fn_secondary_key ...
)]
struct Data(u32, String);

impl Data {
  // Returns primary key as big-endian bytes for consistent lexicographical ordering.
  pub fn p_key(&self) -> Vec<u8> {
    self.0.to_be_bytes().to_vec()
  }

  // Generates a secondary key combining the String field and the big-endian bytes of
  // the primary key for versatile queries.
  pub fn s_key(&self) -> Vec<u8> {
    let mut s_key = self.1.as_bytes().to_vec();
    s_key.extend_from_slice(&self.p_key().as_slice());
    s_key
  }
 }

 fn main() {
  let mut db = Db::init_tmp("my_db_example").unwrap();
  // Initialize the schema
  db.define::<Data>();

  // Insert data
  let txn = db.transaction().unwrap();
  {
    let mut tables = txn.tables();
    tables.insert(&txn, Data(1,"red".to_string())).unwrap();
    tables.insert(&txn, Data(2,"red".to_string())).unwrap();
    tables.insert(&txn, Data(3,"blue".to_string())).unwrap();
  }
  txn.commit().unwrap();
   
  let txn_read = db.read_transaction().unwrap();
  let mut tables = txn_read.tables();
   
  // Retrieve data with p_key=3 
  let retrieve_data: Data = tables.primary_get(&txn_read, &3_u32.to_be_bytes()).unwrap().unwrap();
  println!("data p_key='3' : {:?}", retrieve_data);
   
  // Iterate data with s_key="red" String
  for item in tables.secondary_iter_start_with::<Data>(&txn_read, DataKey::s_key, "red".as_bytes()).unwrap() {
    println!("data s_key='1': {:?}", item);
  }
   
  // Remove data
  let txn = db.transaction().unwrap();
  {
    let mut tables = txn.tables();
    tables.remove(&txn, retrieve_data).unwrap();
  }
  txn.commit().unwrap();
 }
```

# Roadmap

The following features are planned before the 1.0 release

- [ ] Add benchmarks tests. 
- [x] Add documentation.
- [x] Stable release of [redb](https://github.com/cberner/redb) or implement another stable storage engine(s) for Linux, macOS, Windows, Android, iOS.
- [ ] Add support for custom serialization/deserialization logic.
- [x] Add CI for Linux, macOS, Windows, Android, iOS.
- [ ] Use in a real-world project.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/elliot14A"><img src="https://avatars.githubusercontent.com/u/84667163?v=4?s=100" width="100px;" alt="Akshith Madhur"/><br /><sub><b>Akshith Madhur</b></sub></a><br /><a href="https://github.com/vincent-herlemont/struct_db/commits?author=elliot14A" title="Code">💻</a></td>
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