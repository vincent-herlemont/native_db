use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use once_cell::sync::Lazy;
use rand::prelude::SliceRandom;
use redb::{ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

// 1 byte * 10000, 10 bytes * 10000, 100 bytes * 5000, 1KB * 1000, 1MB * 100, 10MB * 10
const ITERATIONS: &'static [(usize, usize)] = &[
    (1, 10000),
    (10, 10000),
    (100, 5000),
    (1024, 1000),
    (1024 * 1024, 100),
    (10 * 1024 * 1024, 10),
];

static DATABASE_BUILDER: Lazy<DatabaseBuilder> = Lazy::new(|| {
    let mut builder = DatabaseBuilder::new();
    builder.define::<Data>().unwrap();
    builder
});

fn init_database() -> (redb::Database, Database<'static>) {
    let redb_backend = redb::backends::InMemoryBackend::new();
    let redb_db = redb::Database::builder()
        .create_with_backend(redb_backend)
        .unwrap();

    let native_db = DATABASE_BUILDER.create_in_memory().unwrap();
    (redb_db, native_db)
}

fn generate_random_data(
    redb_db: &redb::Database,
    native_db: &Database,
    nb_bytes: &usize,
    nb_items: &usize,
) -> Vec<Data> {
    let data = Data {
        x: 1,
        data: vec![1u8; *nb_bytes],
    };

    let mut out = vec![];

    for _ in 0..*nb_items {
        let mut data = data.clone();
        data.random_x();
        use_redb_insert(&redb_db, data.clone());
        use_native_db_insert(&native_db, data.clone());
        out.push(data);
    }

    out
}

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Data {
    #[primary_key]
    x: u32,
    data: Vec<u8>,
}

impl Data {
    fn random_x(&mut self) {
        self.x = rand::random();
    }
}

const TABLE_REDB: TableDefinition<u32, &'static [u8]> = TableDefinition::new("my_data");

fn use_redb_insert(db: &redb::Database, data: Data) {
    let rw = db.begin_write().unwrap();
    {
        let mut table = rw.open_table(TABLE_REDB).unwrap();
        // Because native_db use native_model to encode data, we do the same here
        // to remove the overhead of the encoding.
        let encode = native_model::encode(&data).unwrap();
        table.insert(data.x, encode.as_slice()).unwrap();
    }
    rw.commit().unwrap();
}

fn use_redb_get(db: &redb::Database, x: u32) -> Data {
    let ro = db.begin_read().unwrap();
    let out;
    {
        let table = ro.open_table(TABLE_REDB).unwrap();
        out = table
            .get(x)
            .unwrap()
            .map(|v| native_model::decode(v.value().to_vec()).unwrap().0)
            .expect("Data not found");
    }
    out
}

fn use_redb_scan(db: &redb::Database) -> Vec<Data> {
    let ro = db.begin_read().unwrap();
    let out;
    {
        let table = ro.open_table(TABLE_REDB).unwrap();
        out = table
            .iter()
            .unwrap()
            .map(|r| {
                let (_, v) = r.unwrap();
                native_model::decode(v.value().to_vec()).unwrap().0
            })
            .collect::<Vec<Data>>();
    }
    out
}

fn redb_remove(db: &redb::Database, x: u32) {
    let rw = db.begin_write().unwrap();
    {
        let mut table = rw.open_table(TABLE_REDB).unwrap();
        table.remove(x).unwrap().expect("Data not found");
    }
    rw.commit().unwrap();
}

fn use_native_db_insert(db: &Database, data: Data) {
    let rw = db.rw_transaction().unwrap();
    rw.insert(data).unwrap();
    rw.commit().unwrap();
}

fn use_native_db_scan(db: &Database) -> Vec<Data> {
    let r = db.r_transaction().unwrap();
    let out = r.scan().primary().unwrap().all().try_collect().unwrap();
    out
}

fn use_native_db_get(db: &Database, x: u32) -> Data {
    let r = db.r_transaction().unwrap();
    let out = r.get().primary(x).unwrap().unwrap();
    out
}

fn native_db_remove(db: &Database, data: Data) {
    let rw = db.rw_transaction().unwrap();
    // Remove the old value
    let _ = rw.remove(data).unwrap();
    rw.commit().unwrap();
}

// Benchmarks

fn bench_get_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_random");
    let plot_config =
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic);
    group.plot_config(plot_config.clone());
    group.sampling_mode(criterion::SamplingMode::Flat);

    for (nb_bytes, nb_items) in ITERATIONS {
        group.throughput(criterion::Throughput::Bytes(*nb_bytes as u64));

        let (redb_db, native_db) = init_database();
        let data = generate_random_data(&redb_db, &native_db, nb_bytes, nb_items);

        group.bench_function(BenchmarkId::new("redb", nb_bytes), |b| {
            b.iter_batched(
                || {
                    let item = data.choose(&mut rand::thread_rng()).unwrap();
                    item.x
                },
                |x| use_redb_get(&redb_db, x),
                criterion::BatchSize::SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("native_db", nb_bytes), |b| {
            b.iter_batched(
                || {
                    let item = data.choose(&mut rand::thread_rng()).unwrap();
                    item.x
                },
                |x| use_native_db_get(&native_db, x),
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

fn bench_scan_random(c: &mut Criterion) {
    let plot_config =
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic);
    let mut group = c.benchmark_group("scan_random");
    group.plot_config(plot_config.clone());
    group.sampling_mode(criterion::SamplingMode::Flat);

    for (nb_bytes, nb_items) in ITERATIONS {
        group.throughput(criterion::Throughput::Bytes(*nb_bytes as u64));

        let (redb_db, native_db) = init_database();
        generate_random_data(&redb_db, &native_db, nb_bytes, nb_items);

        group.bench_function(BenchmarkId::new("redb", nb_bytes), |b| {
            b.iter_with_large_drop(|| use_redb_scan(&redb_db));
        });

        group.bench_function(BenchmarkId::new("native_db", nb_bytes), |b| {
            b.iter_with_large_drop(|| use_native_db_scan(&native_db));
        });
    }
}

fn bench_remove_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_random");
    let plot_config =
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic);
    group.plot_config(plot_config.clone());
    group.sampling_mode(criterion::SamplingMode::Flat);

    for (nb_bytes, _nb_items) in ITERATIONS {
        group.throughput(criterion::Throughput::Bytes(*nb_bytes as u64));

        let (redb_db, native_db) = init_database();

        group.bench_function(BenchmarkId::new("redb", nb_bytes), |b| {
            b.iter_batched(
                || {
                    let mut data = Data {
                        x: 1,
                        data: vec![1u8; *nb_bytes as usize],
                    };
                    data.random_x();
                    use_redb_insert(&redb_db, data.clone());
                    data
                },
                |data| redb_remove(&redb_db, data.x),
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("native_db", nb_bytes), |b| {
            b.iter_batched(
                || {
                    let mut data = Data {
                        x: 1,
                        data: vec![1u8; *nb_bytes as usize],
                    };
                    data.random_x();
                    use_native_db_insert(&native_db, data.clone());
                    data
                },
                |data| native_db_remove(&native_db, data),
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

fn bench_insert_random(c: &mut Criterion) {
    let mut insert_random_group = c.benchmark_group("insert_random");
    let plot_config =
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic);
    insert_random_group.plot_config(plot_config.clone());
    insert_random_group.sampling_mode(criterion::SamplingMode::Flat);

    // 1 byte, 10 bytes, 100 bytes, 1KB, 1MB, 10MB
    for (nb_bytes, _) in ITERATIONS {
        insert_random_group.throughput(criterion::Throughput::Bytes(*nb_bytes as u64));

        let data = Data {
            x: 1,
            data: vec![1u8; *nb_bytes as usize],
        };

        let (redb_db, native_db) = init_database();

        let batch_size = match nb_bytes {
            nb_bytes if *nb_bytes < 1024 => criterion::BatchSize::SmallInput,
            nb_bytes if *nb_bytes < 1024 * 1024 => criterion::BatchSize::LargeInput,
            _ => criterion::BatchSize::PerIteration,
        };

        insert_random_group.bench_function(BenchmarkId::new("redb", nb_bytes), |b| {
            b.iter_batched(
                || {
                    let mut data = data.clone();
                    data.random_x();
                    data
                },
                |data| use_redb_insert(&redb_db, data),
                batch_size,
            );
        });

        insert_random_group.bench_function(BenchmarkId::new("native_db", nb_bytes), |b| {
            b.iter_batched(
                || {
                    let mut data = data.clone();
                    data.random_x();
                    data
                },
                |data| use_native_db_insert(&native_db, data),
                batch_size,
            );
        });
    }
    insert_random_group.finish();
}

criterion_group!(
    benches,
    bench_insert_random,
    bench_scan_random,
    bench_get_random,
    bench_remove_random
);
criterion_main!(benches);
