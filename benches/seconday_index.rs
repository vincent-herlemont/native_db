mod setup;
use std::{fmt::Debug, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use native_db::db_type::{KeyDefinition, KeyOptions, ToKeyDefinition};
use rusqlite::TransactionBehavior;
use setup::*;

use rand::Rng;

// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn bench_insert<T: Default + Item + native_db::ToInput>(c: &mut Criterion, item_name: &str) {
    let mut group = c.benchmark_group(format!("insert_{}", item_name));
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    group.bench_function(BenchmarkId::new("XT", "Sqlite"), |b| {
        b.iter_custom(|iters| {
            let sqlite = SqliteBenchDatabase::setup();
            let start = std::time::Instant::now();
            let mut count = 0;
            for _ in 0..iters {
                let mut item = T::default();
                item.update_pk(count);
                sqlite.insert(item);
                count += 1;
            }
            start.elapsed()
        });
    });

    group.bench_function(BenchmarkId::new("1T", "Sqlite"), |b| {
        b.iter_custom(|iters| {
            let sqlite = SqliteBenchDatabase::setup();
            let sqlite = sqlite.db();
            let start = std::time::Instant::now();
            let mut sqlite = sqlite.borrow_mut();
            let transaction = sqlite
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .unwrap();
            let mut count = 0;
            for _ in 0..iters {
                let mut item = T::default();
                item.update_pk(count);
                let binary = item.native_model_encode().unwrap();
                transaction
                    .execute(&item.generate_sqlite_insert(), (binary,))
                    .unwrap();
                count += 1;
            }
            transaction.commit().unwrap();
            start.elapsed()
        });
    });

    group.bench_function(BenchmarkId::new("XT", "Native DB"), |b| {
        b.iter_custom(|iters| {
            let native_db = NativeDBBenchDatabase::setup();
            let start = std::time::Instant::now();
            let mut count = 0;
            for _ in 0..iters {
                let mut item = T::default();
                item.update_pk(count);
                native_db.insert(item);
                count += 1;
            }
            start.elapsed()
        });
    });

    group.bench_function(BenchmarkId::new("1T", "Native DB"), |b| {
        b.iter_custom(|iters| {
            let native_db = NativeDBBenchDatabase::setup();
            let native_db = native_db.db();
            let start = std::time::Instant::now();
            let native_db = native_db.rw_transaction().unwrap();
            let mut count = 0;
            for _ in 0..iters {
                let mut item = T::default();
                item.update_pk(count);
                native_db.insert(item).unwrap();
                count += 1;
            }
            native_db.commit().unwrap();
            start.elapsed()
        });
    });

    group.bench_function(BenchmarkId::new("XT", "Redb"), |b| {
        b.iter_custom(|iters| {
            let redb = RedbBenchDatabase::setup();
            let start = std::time::Instant::now();
            let mut count = 0;
            for _ in 0..iters {
                let mut item = T::default();
                item.update_pk(count);
                redb.insert(item);
                count += 1;
            }
            start.elapsed()
        });
    });

    group.bench_function(BenchmarkId::new("1T", "Redb"), |b| {
        b.iter_custom(|iters| {
            let redb = RedbBenchDatabase::setup();
            let redb = redb.db();
            let start = std::time::Instant::now();
            let rw = redb.begin_write().unwrap();
            {
                let mut table = rw.open_table(REDB_TABLE).unwrap();
                let mut count = 0;
                for _ in 0..iters {
                    let mut item = T::default();
                    item.update_pk(count);
                    let binary = item.native_model_encode().unwrap();
                    table.insert(item.get_pk(), binary).unwrap();
                    count += 1;
                }
            }
            rw.commit().unwrap();
            start.elapsed()
        });
    });
}

struct BenchSelectRangeRandomDataCfg {
    key_def: KeyDefinition<KeyOptions>,
    random: bool,
}

impl BenchSelectRangeRandomDataCfg {
    fn new(key_def: impl ToKeyDefinition<KeyOptions>) -> Self {
        let key_def = key_def.key_definition();
        Self {
            key_def,
            random: false,
        }
    }

    fn random(self) -> Self {
        Self {
            key_def: self.key_def,
            random: true,
        }
    }
}

fn bench_select_range<T: Default + Item + native_db::ToInput + Clone + Debug>(
    c: &mut Criterion,
    item_name: &str,
    cfg: BenchSelectRangeRandomDataCfg,
) {
    let mut group = c.benchmark_group(format!("select_{}", item_name));
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    const NUMBER_OF_ITEMS: usize = 10000;
    const NUMBER_OF_ITEMS_SMALL: usize = NUMBER_OF_ITEMS / 2;
    const FROM_SK_MIN: i64 = 0;
    const FROM_SK_MAX: i64 = 50;
    const TO_SK_MIN: i64 = 50;
    const TO_SK_MAX: i64 = 100;

    let key_def = cfg.key_def.key_definition();

    let function_name  = if cfg.random { "random range" } else { "value range" };

    group.bench_function(BenchmarkId::new(function_name, "Native DB"), |b| {
        b.iter_custom(|iters| {
            let native_db = NativeDBBenchDatabase::setup();
            if cfg.random {
                native_db.insert_bulk_sk_random::<T>(NUMBER_OF_ITEMS);
            } else {
                native_db.insert_bulk_sk_value::<T>(0, NUMBER_OF_ITEMS_SMALL, FROM_SK_MIN);
                native_db.insert_bulk_sk_value::<T>(NUMBER_OF_ITEMS_SMALL as i64, NUMBER_OF_ITEMS_SMALL, TO_SK_MAX);
            }

            let native_db = native_db.db();
            let start = std::time::Instant::now();
            let native_db = native_db.r_transaction().unwrap();
            for _ in 0..iters {
                let (from_sk, to_sk) = if cfg.random {
                    let from_sk: i64 = rand::thread_rng().gen_range(FROM_SK_MIN..FROM_SK_MAX);
                    let to_sk: i64 = rand::thread_rng().gen_range(TO_SK_MIN..TO_SK_MAX);
                    (from_sk, to_sk)
                } else {
                    (FROM_SK_MIN, TO_SK_MIN)
                };
                let _items: Vec<T> = native_db
                    .scan()
                    .secondary(key_def.clone())
                    .unwrap()
                    .range(from_sk..to_sk)
                    .unwrap()
                    .try_collect()
                    .unwrap();
                //println!("Native len: {:?}", _items.len());
            }
            start.elapsed()
        })
    });

    group.bench_function(BenchmarkId::new(function_name, "Sqlite"), |b| {
        b.iter_custom(|iters| {
            let sqlite = SqliteBenchDatabase::setup();
            if cfg.random {
                sqlite.insert_bulk_sk_random::<T>(NUMBER_OF_ITEMS);
            } else {
                sqlite.insert_bulk_sk_value::<T>(0, NUMBER_OF_ITEMS_SMALL, FROM_SK_MIN);
                sqlite.insert_bulk_sk_value::<T>(NUMBER_OF_ITEMS_SMALL as i64, NUMBER_OF_ITEMS_SMALL, TO_SK_MAX);
            }

            let start = std::time::Instant::now();
            for _ in 0..iters {
                let (from_sk, to_sk) = if cfg.random {
                    let from_sk: i64 = rand::thread_rng().gen_range(FROM_SK_MIN..FROM_SK_MAX);
                    let to_sk: i64 = rand::thread_rng().gen_range(TO_SK_MIN..TO_SK_MAX);
                    (from_sk, to_sk)
                } else {
                    (FROM_SK_MIN, TO_SK_MIN)
                };
                let mut db = sqlite.db().borrow_mut();
                let transaction = db
                    .transaction_with_behavior(TransactionBehavior::Immediate)
                    .unwrap();
                let sql = T::generate_select_range_sk(&"sk_1");
                let mut stmt = transaction.prepare(&sql).unwrap();
                let rows = stmt.query_map(&[(":from_sk", &from_sk), (":to_sk", &to_sk)], |row| {
                    let binary: Vec<u8> = row.get(1)?;
                    let item = T::native_db_bincode_decode_from_slice(&binary).unwrap();
                    Ok(item)
                });
                let _out = rows.unwrap().map(|r| r.unwrap()).collect::<Vec<T>>();
                //println!("Sqlite len: {:?}", _out.len());
            }
            start.elapsed()
        });
    });
}

fn bench_get<T: Default + Item + native_db::ToInput + Clone + Debug>(c: &mut Criterion, item_name: &str) {
    let mut group = c.benchmark_group(format!("get_{}", item_name));
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    const NUMBER_OF_ITEMS: usize = 10000;

    group.bench_function(BenchmarkId::new("get", "Native DB"), |b| {
        b.iter_custom(|iters| {
            let native_db = NativeDBBenchDatabase::setup();
            native_db.insert_bulk_inc::<T>(0, NUMBER_OF_ITEMS);

            let native_db = native_db.db();
            let start = std::time::Instant::now();
            let r = native_db.r_transaction().unwrap();
            for _ in 0..iters {
                let pk = rand::thread_rng().gen_range(0..NUMBER_OF_ITEMS as i64);
                let _item: T = r.get().primary(pk).unwrap().unwrap();
            }
            start.elapsed()
        })
    });

    group.bench_function(BenchmarkId::new("get", "Sqlite"), |b| {
        b.iter_custom(|iters| {
            let sqlite = SqliteBenchDatabase::setup();
            sqlite.insert_bulk_inc::<T>(0, NUMBER_OF_ITEMS);

            let start = std::time::Instant::now();
            let db = sqlite.db();
            let mut db = db.borrow_mut();
            let transaction = db
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .unwrap();
            for _ in 0..iters {
                let pk = rand::thread_rng().gen_range(0..NUMBER_OF_ITEMS as i64);
                let sql = T::generate_select_by_pk();
                let mut stmt = transaction.prepare(&sql).unwrap();
                let mut rows = stmt.query(&[(":pk", &pk)]).unwrap();
                let _item: T = if let Some(row) = rows.next().unwrap() {
                    let binary: Vec<u8> = row.get(1).unwrap();
                    Some(T::native_db_bincode_decode_from_slice(&binary).unwrap())
                } else {
                    None
                }.unwrap();
            }
            start.elapsed()
        })
    });

    group.bench_function(BenchmarkId::new("get", "Redb"), |b| {
        b.iter_custom(|iters| {
            let redb = RedbBenchDatabase::setup();
            redb.insert_bulk_inc::<T>(0, NUMBER_OF_ITEMS);

            let redb = redb.db();
            let start = std::time::Instant::now();
            let read_txn = redb.begin_read().unwrap();
            for _ in 0..iters {
                let pk = rand::thread_rng().gen_range(0..NUMBER_OF_ITEMS as i64);
                let table = read_txn.open_table(REDB_TABLE).unwrap();
                let item = table.get(&pk).unwrap();
                let _item: T = item.map(|v| {
                    let bytes = v.value();
                    T::native_db_bincode_decode_from_slice(&bytes).unwrap()
                }).unwrap();
            }
            start.elapsed()
        })
    });
}

fn first_compare(c: &mut Criterion) {
    // bench_insert::<Item1SK_NUni_NOpt>(c, "1 SK no unique no optional");
    // bench_insert::<Item10SK_NUni_NOpt>(c, "10 SK no unique no optional");
    // bench_insert::<Item50SK_NUni_NOpt>(c, "50 SK no unique no optional");
    // bench_insert::<Item100SK_NUni_NOpt>(c, "100 SK no unique no optional");

    bench_get::<Item1SK_NUni_NOpt>(c, "1 PK no unique no optional");
    bench_get::<Item10SK_NUni_NOpt>(c, "10 PK no unique no optional");
    bench_get::<Item50SK_NUni_NOpt>(c, "50 PK no unique no optional");
    bench_get::<Item100SK_NUni_NOpt>(c, "100 PK no unique no optional");

    // // Range
    // bench_select_range::<Item1SK_NUni_NOpt>(
    //     c,
    //     "1 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item1SK_NUni_NOptKey::sk_1),
    // );
    // bench_select_range::<Item10SK_NUni_NOpt>(
    //     c,
    //     "10 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item10SK_NUni_NOptKey::sk_1),
    // );
    // bench_select_range::<Item50SK_NUni_NOpt>(
    //     c,
    //     "50 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item50SK_NUni_NOptKey::sk_1),
    // );
    // bench_select_range::<Item100SK_NUni_NOpt>(
    //     c,
    //     "100 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item100SK_NUni_NOptKey::sk_1),
    // );

    // // Range random
    // bench_select_range::<Item1SK_NUni_NOpt>(
    //     c,
    //     "1 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item1SK_NUni_NOptKey::sk_1).random(),
    // );
    // bench_select_range::<Item10SK_NUni_NOpt>(
    //     c,
    //     "10 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item10SK_NUni_NOptKey::sk_1).random(),
    // );
    // bench_select_range::<Item50SK_NUni_NOpt>(
    //     c,
    //     "50 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item50SK_NUni_NOptKey::sk_1).random(),
    // );
    // bench_select_range::<Item100SK_NUni_NOpt>(
    //     c,
    //     "100 SK no unique no optional",
    //     BenchSelectRangeRandomDataCfg::new(Item100SK_NUni_NOptKey::sk_1).random(),
    // );
}

fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        // 5 minutes
        // .measurement_time(Duration::from_secs(300))
}

criterion_group!(
    name = benches;
    config = configure_criterion();
    targets = first_compare
);
criterion_main!(benches);
