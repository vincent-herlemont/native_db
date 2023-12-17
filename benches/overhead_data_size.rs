use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use native_db::*;
use native_model::{native_model, Model};
use redb::TableDefinition;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Data {
    #[primary_key]
    x: u32,
    data: Vec<u8>,
}

const TABLE_REDB: TableDefinition<u32, &'static [u8]> = TableDefinition::new("my_data");
fn use_redb(db: &redb::Database, data: Data) {
    let rw = db.begin_write().unwrap();
    {
        let mut table = rw.open_table(TABLE_REDB).unwrap();
        let encode = native_model::encode(&data).unwrap();
        table.insert(data.x, encode.as_slice()).unwrap();
    }
    rw.commit().unwrap();
}

fn use_native_db(db: &native_db::Database, data: Data) {
    let rw = db.rw_transaction().unwrap();
    rw.insert(data).unwrap();
    rw.commit().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    // 1 byte, 1KB, 1MB, 10MB, 100MB
    for nb_bytes in [1, 1024, 1024 * 1024, 10 * 1024 * 1024, 100 * 1024 * 1024] {
        group.throughput(criterion::Throughput::Bytes(nb_bytes as u64));

        let data = Data {
            x: 1,
            data: vec![1u8; nb_bytes as usize],
        };

        let redb_backend = redb::backends::InMemoryBackend::new();
        let redb_db = redb::Database::builder()
            .create_with_backend(redb_backend)
            .unwrap();

        group.bench_function(BenchmarkId::new("redb", nb_bytes), |b| {
            b.iter(|| use_redb(&redb_db, data.clone()))
        });

        let mut native_db = native_db::Database::create_in_memory().unwrap();
        native_db.define::<Data>().unwrap();
        group.bench_function(BenchmarkId::new("native_db", nb_bytes), |b| {
            b.iter(|| use_native_db(&native_db, data.clone()))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
