mod setup;
use std::{fmt::Debug, time::Duration};

use rusqlite::TransactionBehavior;
use setup::*;
use itertools::Itertools;
use criterion::{
    criterion_group, criterion_main, BenchmarkId,
    Criterion,
};
use native_db::db_type::ToKeyDefinition;

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

fn bench_select_range_random_data<T: Default + Item + native_db::ToInput + Clone + Debug>(
    c: &mut Criterion,
    item_name: &str,
    key_def: impl ToKeyDefinition<native_db::db_type::KeyOptions>,
) {
    let mut group = c.benchmark_group(format!("select_{}", item_name));
    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear),
    );
    group.sampling_mode(criterion::SamplingMode::Flat);

    const NUMBER_OF_ITEMS: usize = 10000;

    let key_def = key_def.key_definition();

    group.bench_function(BenchmarkId::new("random range", "Native DB"), |b| {
        b.iter_custom(|iters| {
            let native_db = NativeDBBenchDatabase::setup();
            native_db.insert_bulk_random::<T>(NUMBER_OF_ITEMS);

            let native_db = native_db.db();
            let start = std::time::Instant::now();
            let native_db = native_db.r_transaction().unwrap();
            for _ in 0..iters {
                let from_sk: i64 = rand::thread_rng().gen_range(0..50);
                let to_sk: i64 = rand::thread_rng().gen_range(50..100);
                let _items: Vec<T> = native_db.scan().secondary(key_def.clone()).unwrap().range(from_sk..to_sk).unwrap().try_collect().unwrap();
                // println!("len: {:?}", _items.len());
            }
            start.elapsed()
        })
    });

    
    group.bench_function(BenchmarkId::new("random range", "Sqlite"), |b| {
        b.iter_custom(|iters| {
            let sqlite = SqliteBenchDatabase::setup();
            sqlite.insert_bulk_random::<T>(NUMBER_OF_ITEMS);
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let from_sk: i64 = rand::thread_rng().gen_range(0..50);
                let to_sk: i64 = rand::thread_rng().gen_range(50..100);
                let mut db = sqlite.db().borrow_mut();
                let transaction = db
                    .transaction_with_behavior(TransactionBehavior::Immediate)
                    .unwrap();
                let sql = T::generate_select_range_sk(&"sk_1");
                let mut stmt = transaction.prepare_cached(&sql).unwrap();
                let rows = stmt.query_map(&[(":from_sk", &from_sk), (":to_sk", &to_sk)], |row| {
                    let binary: Vec<u8> = row.get(1)?;
                    let item = T::native_db_bincode_decode_from_slice(&binary).unwrap();
                    Ok(item)
                });
                let _out = rows.unwrap().map(|r| r.unwrap()).collect::<Vec<T>>();
                // println!("len: {:?}", _out.len());
            }
            start.elapsed()
        });
    });

}

fn first_compare(c: &mut Criterion) {
    // bench_insert::<Item1SK_NUni_NOpt>(c, "1 SK no unique no optional");
    // bench_insert::<Item10SK_NUni_NOpt>(c, "10 SK no unique no optional");
    // bench_insert::<Item50SK_NUni_NOpt>(c, "50 SK no unique no optional");
    // bench_insert::<Item100SK_NUni_NOpt>(c, "100 SK no unique no optional");

    // TODO update

    // TODO get once

    // TODO: range with no random data
    // bench_select_range

    bench_select_range_random_data::<Item1SK_NUni_NOpt>(c, "1 SK no unique no optional", Item1SK_NUni_NOptKey::sk_1);
    bench_select_range_random_data::<Item10SK_NUni_NOpt>(c, "10 SK no unique no optional", Item10SK_NUni_NOptKey::sk_1);
    bench_select_range_random_data::<Item50SK_NUni_NOpt>(c, "50 SK no unique no optional", Item50SK_NUni_NOptKey::sk_1);
    bench_select_range_random_data::<Item100SK_NUni_NOpt>(c, "100 SK no unique no optional", Item100SK_NUni_NOptKey::sk_1);

    // TODO delete

    // TODO: insert select update concurently
}

fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(5))
}

criterion_group!(
    name = benches;
    config = configure_criterion();
    targets = first_compare
);
criterion_main!(benches);
