mod setup;
use std::fmt::Debug;

use rusqlite::TransactionBehavior;
use setup::*;
use itertools::Itertools;
use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId,
    Criterion,
};
use native_model::{native_model, Model};
use rand::Rng;

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

fn bench_select<T: Default + Item + native_db::ToInput + Clone + Debug>(
    c: &mut Criterion,
    item_name: &str,
) {
    let mut group = c.benchmark_group(format!("select_{}", item_name));
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    const NUMBER_OF_ITEMS: usize = 10000;

    group.bench_function(BenchmarkId::new("random range", "Native DB"), |b| {
        b.iter_custom(|iters| {
            let native_db = NativeDBBenchDatabase::setup();
            native_db.insert_bulk_random::<T>(NUMBER_OF_ITEMS);

            let native_db = native_db.db();
            let start = std::time::Instant::now();
            let native_db = native_db.r_transaction().unwrap();
            for _ in 0..iters {
                let from_sk = 1;
                let to_sk = 100;
                let items: Vec<T> = native_db.scan().secondary(Item1SK_NUni_NOptKey::sk_1).unwrap().range(from_sk..).unwrap().try_collect().unwrap();
                println!("from_sk: {:?}, to_sk: {:?}, len: {:?}", from_sk, to_sk, items.len());
            }
            start.elapsed()
        })
    });


    // group.bench_function(BenchmarkId::new("random range", "Sqlite"), |b| {
    //     b.iter_custom(|iters| {
    //         let sqlite = SqliteBenchDatabase::setup();
    //         sqlite.insert_bulk_random::<T>(NUMBER_OF_ITEMS);
    //         let start = std::time::Instant::now();
    //         for _ in 0..iters {
    //             let from_sk = rand::thread_rng().gen_range(0..50);
    //             let to_sk = rand::thread_rng().gen_range(50..100);
    //             let mut db = sqlite.db().borrow_mut();
    //             let transaction = db
    //                 .transaction_with_behavior(TransactionBehavior::Immediate)
    //                 .unwrap();
    //             let sql = T::generate_select_range_sk(&"sk_1");
    //             let mut stmt = transaction.prepare_cached(&sql).unwrap();
    //             let rows = stmt.query_map(&[(":from_sk", &from_sk), (":to_sk", &to_sk)], |row| {
    //                 let binary: Vec<u8> = row.get(1)?;
    //                 let item = T::native_db_bincode_decode_from_slice(&binary).unwrap();
    //                 Ok(item)
    //             });
    //             let out = rows.unwrap().map(|r| r.unwrap()).collect::<Vec<T>>();
    //             println!("len: {:?}", out.len());
    //         }
    //         start.elapsed()
    //     });
    // });
}

fn first_compare(c: &mut Criterion) {
    // bench_insert::<Item1SK_NUni_NOpt>(c, "1 SK no unique no optional");
    // bench_insert::<Item10SK_NUni_NOpt>(c, "10 SK no unique no optional");
    // bench_insert::<Item50SK_NUni_NOpt>(c, "50 SK no unique no optional");
    // bench_insert::<Item100SK_NUni_NOpt>(c, "100 SK no unique no optional");

    bench_select::<Item1SK_NUni_NOpt>(c, "1 SK no unique no optional");
}

criterion_group!(benches, first_compare);
criterion_main!(benches);
