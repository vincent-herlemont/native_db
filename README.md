# Struct DB 🔧🔩 

[![Crates.io](https://img.shields.io/crates/v/struct_db?style=flat-square)](https://crates.io/crates/struct_db)

Goal: Embedded database that maintains coherence between Rust types and stored data with minimal boilerplate, enjoy 😌🍃.

# Features

- Fast as the storage engine you choose ([redb](https://github.com/cberner/redb), _[IndexDB](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.IdbDatabase.html) [planned*](#roadmap)_).
- Embedded database (Linux, macOS, Windows, Android, iOS, _WebBrowser WASM [planned*](#roadmap)_).
- Support multiple indexes (unique secondary keys).
- Compatible with all Rust types (`enum`, `struct`, `tuple` etc.).
- Query data (`get`, `watch`, `iter` etc.) using explicit type or type inference. 
- Real-time subscription with filters for `insert`, `update` and `delete` operations.
- Schema migration using native Rust coercion.
- Fully ACID-compliant transactions.
- _Add your own serialization/deserialization logic [planned*](#roadmap) (e.g: zero-copy)._
- Thread-safe.
- Plus, all features depending on the storage engine you choose:
   - [redb](https://github.com/cberner/redb) support: Linux, macOS, Windows, Android, iOS.
   - _[IndexDB](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.IdbDatabase.html) support: WebBrowser (WASM) [planned*](#roadmap)._
   - Open an issue if you want to add another storage engine.

# Status

Early development. Not ready for production.

# How to use?

See [docs.rs](https://docs.rs/struct_db/latest/struct_db/).

# Example

```rust
use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[struct_db(
    fn_primary_key(p_key),
    fn_secondary_key(s_key),
)]
struct Data(u32, String);

impl Data {
    // `p_key` returns the primary key of the `Data` struct as a vector of bytes.
    // In this case, it is the big-endian byte representation of the `i32` value.
    // Using big-endian representation for the primary key maintains a consistent
    // lexicographical ordering of the keys, which is useful for ordered key-value
    // stores and efficient range queries.
   pub fn p_key(&self) -> Vec<u8> {
       self.0.to_be_bytes().to_vec()
   }
  
    // `s_key` generates a secondary key for the `Data` struct as a vector of bytes.
    // The secondary key consists of the big-endian byte representation of the `i32` value
    // (the primary key) followed by the String field. This combined key allows for more
    // versatile querying options.
   pub fn s_key(&self) -> Vec<u8> {
       let mut p_key = self.p_key();
       p_key.extend(self.1.as_bytes());
       p_key
   }
}

fn main() {
    let mut db = Db::init_tmp("my_db").unwrap();
    // Initialize the schema
    db.add_schema(Data::struct_db_schema());

    let data = Data(1,"test".to_string());
    // Insert data
    let txn = db.transaction().unwrap();
    {
      let mut tables = txn.tables();
      tables.insert(&txn, data).unwrap();
    }
    txn.commit().unwrap();

    // Get data
    let txn_read = db.read_transaction().unwrap();
    let retrieve_data: Data = txn_read.tables().primary_get(&txn_read, &1_u32.to_be_bytes()).unwrap().unwrap();
    assert_eq!(&retrieve_data, &Data(1,"test".to_string()));
  
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

- Stabilize the wording, if you have any suggestion follow [this issue](https://github.com/vincent-herlemont/struct_db/issues/1) 🙏.
- Add benchmarks tests.
- Add documentation.
- Stable release of [redb](https://github.com/cberner/redb) or implement another stable storage engine(s) for Linux, macOS, Windows, Android, iOS.
- Add support for [IndexDB](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.IdbDatabase.html) (WebBrowser).
- Add support for custom serialization/deserialization logic.



