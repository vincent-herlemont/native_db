mod setup;
use std::{fmt::Debug, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use native_db::db_type::{KeyDefinition, KeyOptions, ToKeyDefinition};
use rusqlite::TransactionBehavior;
use setup::*;

use rand::Rng;

const DB_NAME_REDB: &str = "Redb";
const DB_NAME_SQLITE: &str = "Sqlite";
const DB_NAME_NATIVE_DB: &str = "Native_db";
const DB_NAME_NATIVE_DB_TWO_PHASE_COMMIT: &str = "Native_db_twophasecommit";
const DB_NAME_NATIVE_DB_QUICK_REPAIR: &str = "Native_db_quickrepair";

fn bench_insert<T: Default + Item + native_db::ToInput>(
    c: &mut Criterion,
    bench_display: BenchDisplay,
) {
    let mut group = c.benchmark_group("Insert");
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    let name_to_mode = [
        (DB_NAME_NATIVE_DB, &Mode::Default),
        (DB_NAME_NATIVE_DB_TWO_PHASE_COMMIT, &Mode::TwoPhaseCommit),
        (DB_NAME_NATIVE_DB_QUICK_REPAIR, &Mode::QuickRepair)
    ];

    for (name, mode) in name_to_mode {
        group.bench_function(
            BenchmarkId::new(name, bench_display.display_n_by_tranaction()),
            |b| {
                b.iter_custom(|iters| {
                    let mut native_db = NativeDBBenchDatabase::setup();
                    native_db.set_mode(mode);
                    let native_db = native_db.db();
                    let start = std::time::Instant::now();
                    let mut native_db = native_db.rw_transaction().unwrap();
                    native_db.set_two_phase_commit(true);
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
            },
        );
    }

    if bench_display == BenchDisplay::SK_1 {
        group.bench_function(
            BenchmarkId::new(DB_NAME_REDB, bench_display.display_n_by_tranaction()),
            |b| {
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
            },
        );
    }

    group.bench_function(
        BenchmarkId::new(DB_NAME_SQLITE, bench_display.display_n_by_tranaction()),
        |b| {
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
        },
    );

    for (name, mode) in name_to_mode {
        group.bench_function(
            BenchmarkId::new(name, bench_display.display_1_by_tranaction()),
            |b| {
                b.iter_custom(|iters| {
                    let mut native_db = NativeDBBenchDatabase::setup();
                    native_db.set_mode(mode);
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
            },
        );
    }

    if bench_display == BenchDisplay::SK_1 {
        group.bench_function(
            BenchmarkId::new(DB_NAME_REDB, bench_display.display_1_by_tranaction()),
            |b| {
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
            },
        );
    }

    group.bench_function(
        BenchmarkId::new(DB_NAME_SQLITE, bench_display.display_1_by_tranaction()),
        |b| {
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
        },
    );
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
    bench_display: BenchDisplay,
    cfg: BenchSelectRangeRandomDataCfg,
) {
    let mut group = c.benchmark_group("Select Range Secondary Key".to_string());
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

    let title_random_or_value = if cfg.random {
        "random range"
    } else {
        "value range"
    };

    let name_to_mode = [
        (DB_NAME_NATIVE_DB, &Mode::Default),
        (DB_NAME_NATIVE_DB_TWO_PHASE_COMMIT, &Mode::TwoPhaseCommit),
        (DB_NAME_NATIVE_DB_QUICK_REPAIR, &Mode::QuickRepair)
    ];

    for (name, mode) in name_to_mode {
        group.bench_function(
            BenchmarkId::new(
                name,
                bench_display.display_read_custom(title_random_or_value),
            ),
            |b| {
                b.iter_custom(|iters| {
                    let mut native_db = NativeDBBenchDatabase::setup();
                    native_db.set_mode(mode);
                    if cfg.random {
                        native_db.insert_bulk_sk_random::<T>(NUMBER_OF_ITEMS);
                    } else {
                        native_db.insert_bulk_sk_value::<T>(0, NUMBER_OF_ITEMS_SMALL, FROM_SK_MIN);
                        native_db.insert_bulk_sk_value::<T>(
                            NUMBER_OF_ITEMS_SMALL as i64,
                            NUMBER_OF_ITEMS_SMALL,
                            TO_SK_MAX,
                        );
                    }

                    let native_db = native_db.db();
                    let start = std::time::Instant::now();
                    let native_db = native_db.r_transaction().unwrap();
                    for _ in 0..iters {
                        let (from_sk, to_sk) = if cfg.random {
                            let from_sk: i64 = rand::rng().random_range(FROM_SK_MIN..FROM_SK_MAX);
                            let to_sk: i64 = rand::rng().random_range(TO_SK_MIN..TO_SK_MAX);
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
            },
        );
    }

    group.bench_function(
        BenchmarkId::new(
            DB_NAME_SQLITE,
            bench_display.display_read_custom(title_random_or_value),
        ),
        |b| {
            b.iter_custom(|iters| {
                let sqlite = SqliteBenchDatabase::setup();
                if cfg.random {
                    sqlite.insert_bulk_sk_random::<T>(NUMBER_OF_ITEMS);
                } else {
                    sqlite.insert_bulk_sk_value::<T>(0, NUMBER_OF_ITEMS_SMALL, FROM_SK_MIN);
                    sqlite.insert_bulk_sk_value::<T>(
                        NUMBER_OF_ITEMS_SMALL as i64,
                        NUMBER_OF_ITEMS_SMALL,
                        TO_SK_MAX,
                    );
                }

                let start = std::time::Instant::now();
                for _ in 0..iters {
                    let (from_sk, to_sk) = if cfg.random {
                        let from_sk: i64 = rand::rng().random_range(FROM_SK_MIN..FROM_SK_MAX);
                        let to_sk: i64 = rand::rng().random_range(TO_SK_MIN..TO_SK_MAX);
                        (from_sk, to_sk)
                    } else {
                        (FROM_SK_MIN, TO_SK_MIN)
                    };
                    let mut db = sqlite.db().borrow_mut();
                    let transaction = db
                        .transaction_with_behavior(TransactionBehavior::Immediate)
                        .unwrap();
                    let sql = T::generate_select_range_sk("sk_1");
                    let mut stmt = transaction.prepare(&sql).unwrap();
                    let rows =
                        stmt.query_map(&[(":from_sk", &from_sk), (":to_sk", &to_sk)], |row| {
                            let binary: Vec<u8> = row.get(1)?;
                            let item = T::native_db_bincode_decode_from_slice(&binary).unwrap();
                            Ok(item)
                        });
                    let _out = rows.unwrap().map(|r| r.unwrap()).collect::<Vec<T>>();
                    //println!("Sqlite len: {:?}", _out.len());
                }
                start.elapsed()
            });
        },
    );
}

fn bench_get<T: Default + Item + native_db::ToInput + Clone + Debug>(
    c: &mut Criterion,
    bench_display: BenchDisplay,
) {
    let mut group = c.benchmark_group("Get".to_string());
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    const NUMBER_OF_ITEMS: usize = 10000;

    let name_to_mode = [
        (DB_NAME_NATIVE_DB, &Mode::Default),
        (DB_NAME_NATIVE_DB_TWO_PHASE_COMMIT, &Mode::TwoPhaseCommit),
        (DB_NAME_NATIVE_DB_QUICK_REPAIR, &Mode::QuickRepair)
    ];

    for (name, mode) in name_to_mode {
        group.bench_function(
            BenchmarkId::new(name, bench_display.display_read()),
            |b| {
                b.iter_custom(|iters| {
                    let mut native_db = NativeDBBenchDatabase::setup();
                    native_db.set_mode(mode);
                    native_db.insert_bulk_inc::<T>(0, NUMBER_OF_ITEMS);

                    let native_db = native_db.db();
                    let start = std::time::Instant::now();
                    let r = native_db.r_transaction().unwrap();
                    for _ in 0..iters {
                        let pk = rand::rng().random_range(0..NUMBER_OF_ITEMS as i64);
                        let _item: T = r.get().primary(pk).unwrap().unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }

    if bench_display == BenchDisplay::SK_1 {
        group.bench_function(
            BenchmarkId::new(DB_NAME_REDB, bench_display.display_read()),
            |b| {
                b.iter_custom(|iters| {
                    let redb = RedbBenchDatabase::setup();
                    redb.insert_bulk_inc::<T>(0, NUMBER_OF_ITEMS);

                    let redb = redb.db();
                    let start = std::time::Instant::now();
                    let read_txn = redb.begin_read().unwrap();
                    for _ in 0..iters {
                        let pk = rand::rng().random_range(0..NUMBER_OF_ITEMS as i64);
                        let table = read_txn.open_table(REDB_TABLE).unwrap();
                        let item = table.get(&pk).unwrap();
                        let _item: T = item
                            .map(|v| {
                                let bytes = v.value();
                                T::native_db_bincode_decode_from_slice(&bytes).unwrap()
                            })
                            .unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }

    group.bench_function(
        BenchmarkId::new(DB_NAME_SQLITE, bench_display.display_read()),
        |b| {
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
                    let pk = rand::rng().random_range(0..NUMBER_OF_ITEMS as i64);
                    let sql = T::generate_select_by_pk();
                    let mut stmt = transaction.prepare(&sql).unwrap();
                    let mut rows = stmt.query(&[(":pk", &pk)]).unwrap();
                    let _item: T = if let Some(row) = rows.next().unwrap() {
                        let binary: Vec<u8> = row.get(1).unwrap();
                        Some(T::native_db_bincode_decode_from_slice(&binary).unwrap())
                    } else {
                        None
                    }
                    .unwrap();
                }
                start.elapsed()
            })
        },
    );
}

fn bench_delete<T: Default + Item + native_db::ToInput + Clone + Debug>(
    c: &mut Criterion,
    bench_display: BenchDisplay,
) {
    let mut group = c.benchmark_group("Delete".to_string());
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    let name_to_mode = [
        (DB_NAME_NATIVE_DB, &Mode::Default),
        (DB_NAME_NATIVE_DB_TWO_PHASE_COMMIT, &Mode::TwoPhaseCommit),
        (DB_NAME_NATIVE_DB_QUICK_REPAIR, &Mode::QuickRepair)
    ];

    for (name, mode) in name_to_mode.clone() {
        group.bench_function(
            BenchmarkId::new(name, bench_display.display_n_by_tranaction()),
            |b| {
                b.iter_custom(|iters| {
                    let mut native_db = NativeDBBenchDatabase::setup();
                    native_db.set_mode(mode);
                    let items = native_db.insert_bulk_inc::<T>(0, iters as usize);

                    let native_db = native_db.db();
                    let start = std::time::Instant::now();
                    let w = native_db.rw_transaction().unwrap();
                    for item in items {
                        w.remove(item).unwrap();
                    }
                    w.commit().unwrap();
                    start.elapsed()
                })
            },
        );
    }

    if bench_display == BenchDisplay::SK_1 {
        group.bench_function(
            BenchmarkId::new(DB_NAME_REDB, bench_display.display_n_by_tranaction()),
            |b| {
                b.iter_custom(|iters| {
                    let redb = RedbBenchDatabase::setup();
                    let items = redb.insert_bulk_inc::<T>(0, iters as usize);

                    let redb = redb.db();
                    let start = std::time::Instant::now();
                    let write_txn = redb.begin_write().unwrap();
                    {
                        let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                        for item in items {
                            let pk = item.get_pk();
                            table.remove(&pk).unwrap();
                        }
                    }
                    write_txn.commit().unwrap();
                    start.elapsed()
                });
            },
        );
    }

    group.bench_function(
        BenchmarkId::new(DB_NAME_SQLITE, bench_display.display_n_by_tranaction()),
        |b| {
            b.iter_custom(|iters| {
                let sqlite = SqliteBenchDatabase::setup();
                let items = sqlite.insert_bulk_inc::<T>(0, iters as usize);

                let start = std::time::Instant::now();
                let db = sqlite.db();
                let mut db = db.borrow_mut();
                let transaction = db
                    .transaction_with_behavior(TransactionBehavior::Immediate)
                    .unwrap();
                for item in items {
                    let pk = item.get_pk();
                    let sql = T::generate_delete_by_pk();
                    let mut stmt = transaction.prepare(&sql).unwrap();
                    stmt.execute(&[(":pk", &pk)]).unwrap();
                }
                transaction.commit().unwrap();
                start.elapsed()
            });
        },
    );

    for (name, mode) in name_to_mode {
        group.bench_function(
            BenchmarkId::new(name, bench_display.display_1_by_tranaction()),
            |b| {
                b.iter_custom(|iters| {
                    let mut native_db = NativeDBBenchDatabase::setup();
                    native_db.set_mode(mode);
                    let items = native_db.insert_bulk_inc::<T>(0, iters as usize);

                    let native_db = native_db.db();
                    let start = std::time::Instant::now();
                    for item in items {
                        let w = native_db.rw_transaction().unwrap();
                        w.remove(item).unwrap();
                        w.commit().unwrap();
                    }
                    start.elapsed()
                })
            },
        );
    }

    if bench_display == BenchDisplay::SK_1 {
        group.bench_function(
            BenchmarkId::new(DB_NAME_REDB, bench_display.display_1_by_tranaction()),
            |b| {
                b.iter_custom(|iters| {
                    let redb = RedbBenchDatabase::setup();
                    let items = redb.insert_bulk_inc::<T>(0, iters as usize);

                    let redb = redb.db();
                    let start = std::time::Instant::now();
                    for item in items {
                        let write_txn = redb.begin_write().unwrap();
                        {
                            let mut table = write_txn.open_table(REDB_TABLE).unwrap();
                            let pk = item.get_pk();
                            table.remove(&pk).unwrap();
                        }
                        write_txn.commit().unwrap();
                    }
                    start.elapsed()
                });
            },
        );
    }

    group.bench_function(
        BenchmarkId::new(DB_NAME_SQLITE, bench_display.display_1_by_tranaction()),
        |b| {
            b.iter_custom(|iters| {
                let sqlite = SqliteBenchDatabase::setup();
                let items = sqlite.insert_bulk_inc::<T>(0, iters as usize);

                let start = std::time::Instant::now();
                let db = sqlite.db();
                for item in items {
                    let mut db = db.borrow_mut();
                    let transaction = db
                        .transaction_with_behavior(TransactionBehavior::Immediate)
                        .unwrap();
                    {
                        let pk = item.get_pk();
                        let sql = T::generate_delete_by_pk();
                        let mut stmt = transaction.prepare(&sql).unwrap();
                        stmt.execute(&[(":pk", &pk)]).unwrap();
                    }
                    transaction.commit().unwrap();
                }
                start.elapsed()
            });
        },
    );
}

fn run_all(c: &mut Criterion) {
    // Insert
    bench_insert::<Item1SK_NUni_NOpt>(c, BenchDisplay::SK_1);
    bench_insert::<Item10SK_NUni_NOpt>(c, BenchDisplay::SK_10);
    bench_insert::<Item50SK_NUni_NOpt>(c, BenchDisplay::SK_50);
    bench_insert::<Item100SK_NUni_NOpt>(c, BenchDisplay::SK_100);

    // Get
    bench_get::<Item1SK_NUni_NOpt>(c, BenchDisplay::SK_1);
    bench_get::<Item10SK_NUni_NOpt>(c, BenchDisplay::SK_10);
    bench_get::<Item50SK_NUni_NOpt>(c, BenchDisplay::SK_50);
    bench_get::<Item100SK_NUni_NOpt>(c, BenchDisplay::SK_100);

    // Range
    bench_select_range::<Item1SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_1,
        BenchSelectRangeRandomDataCfg::new(Item1SK_NUni_NOptKey::sk_1),
    );
    bench_select_range::<Item10SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_10,
        BenchSelectRangeRandomDataCfg::new(Item10SK_NUni_NOptKey::sk_1),
    );
    bench_select_range::<Item50SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_50,
        BenchSelectRangeRandomDataCfg::new(Item50SK_NUni_NOptKey::sk_1),
    );
    bench_select_range::<Item100SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_100,
        BenchSelectRangeRandomDataCfg::new(Item100SK_NUni_NOptKey::sk_1),
    );
    // Range random
    bench_select_range::<Item1SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_1,
        BenchSelectRangeRandomDataCfg::new(Item1SK_NUni_NOptKey::sk_1).random(),
    );
    bench_select_range::<Item10SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_10,
        BenchSelectRangeRandomDataCfg::new(Item10SK_NUni_NOptKey::sk_1).random(),
    );
    bench_select_range::<Item50SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_50,
        BenchSelectRangeRandomDataCfg::new(Item50SK_NUni_NOptKey::sk_1).random(),
    );
    bench_select_range::<Item100SK_NUni_NOpt>(
        c,
        BenchDisplay::SK_100,
        BenchSelectRangeRandomDataCfg::new(Item100SK_NUni_NOptKey::sk_1).random(),
    );

    // Delete
    bench_delete::<Item1SK_NUni_NOpt>(c, BenchDisplay::SK_1);
    bench_delete::<Item10SK_NUni_NOpt>(c, BenchDisplay::SK_10);
    bench_delete::<Item50SK_NUni_NOpt>(c, BenchDisplay::SK_50);
    bench_delete::<Item100SK_NUni_NOpt>(c, BenchDisplay::SK_100);
}

fn configure_criterion() -> Criterion {
    Criterion::default()
        // .sample_size(10)
        // 5 minutes
        .measurement_time(Duration::from_secs(300))
}

criterion_group!(
    name = benches;
    config = configure_criterion();
    targets = run_all
);
criterion_main!(benches);
