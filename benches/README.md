# Benchmarks

## Table of Contents

- [Overview](#overview)
- [Benchmark Results](#benchmark-results)
    - [Insert](#insert)
    - [Get](#get)
    - [Select Range Secondary Key](#select-range-secondary-key)
    - [Delete](#delete)

## Overview


- :warning: This benchmark is an initial version and it can certainly be greatly improved to make the results as relevant as possible. Feel free to open issues to improve it. 
- :point_right: Native DB will be further improved in the future as performance issues have not yet been addressed. That is indeed the purpose of this benchmark, which is to provide visibility on what needs to be improved.

Comparison between [`Native DB`](https://github.com/vincent-herlemont/native_db) vs [`Redb`](https://github.com/cberner/redb) vs [`SQLite`](https://www.sqlite.org/)

- Why compare with `Redb`?
  - To highlight the `Native DB` overhead, because `Redb` is the backend of `Native DB`, it should "normally" always be faster than `Native DB`.
- Why compare with `SQLite`?
  - Because even though `SQLite` offers a lot more options, `Native DB` can be seen as a very light alternative to `SQLite`.
- And the other databases?
  - Knowing the capabilities of `Native DB` compared to `Redb` with the benchmark below, you can check the benchmark of redb here: [cberner/redb/benchmarks](https://github.com/cberner/redb?tab=readme-ov-file#benchmarks)

The benchmarks ignore:
 - [`native_model`](https://github.com/vincent-herlemont/native_model) overhead.
 - Serialization overhead used by `native_model` like `bincode`,`postcard` etc.
 - The fact that `redb` can write the data using zero-copy.

Explanation:
 - `1:SK`, `10:SK`, `50:SK`, `100:SK`, `N:SK` in this case `N` is the number of secondary keys (`SK`) for the same data. Regarding SQLite, it is the column with each having a secondary index.
 - `1:T`, `n:T` represent the number of operations per transaction.
   - `1:T` means one operation per transaction, for example, for insertion, it means there is only one insert operation per transaction.
   - `n:T` means `n` operations per transaction, `n` is defined by criteria, meaning that all operations are within a single transaction.
 - We can see that `Redb` sometimes has no comparisons (`N/A`) because `Redb` is a key-value database and does not support secondary indexes. Therefore, it is pointless to compare with more or fewer secondary indexes.

## Benchmark Results

### Insert

|                       | `Native_db`               | `Redb`                           | `Sqlite`                           |
|:----------------------|:--------------------------|:---------------------------------|:---------------------------------- |
| **`1:SK with n:T`**   | `3.91 us` (✅ **1.00x**)   | `960.85 ns` (🚀 **4.07x faster**) | `1.11 us` (🚀 **3.53x faster**)     |
| **`1:SK with 1:T`**   | `4.39 ms` (✅ **1.00x**)   | `4.15 ms` (✅ **1.06x faster**)   | `477.93 us` (🚀 **9.19x faster**)   |
| **`10:SK with n:T`**  | `24.20 us` (✅ **1.00x**)  | `N/A`                            | `2.67 us` (🚀 **9.07x faster**)     |
| **`10:SK with 1:T`**  | `4.39 ms` (✅ **1.00x**)   | `N/A`                            | `496.44 us` (🚀 **8.85x faster**)   |
| **`50:SK with n:T`**  | `114.81 us` (✅ **1.00x**) | `N/A`                            | `12.74 us` (🚀 **9.01x faster**)    |
| **`50:SK with 1:T`**  | `5.69 ms` (✅ **1.00x**)   | `N/A`                            | `525.59 us` (🚀 **10.83x faster**)  |
| **`100:SK with n:T`** | `226.27 us` (✅ **1.00x**) | `N/A`                            | `36.28 us` (🚀 **6.24x faster**)    |
| **`100:SK with 1:T`** | `6.81 ms` (✅ **1.00x**)   | `N/A`                            | `557.70 us` (🚀 **12.20x faster**)  |

### Get

|              | `Native_db`               | `Redb`                           | `Sqlite`                         |
|:-------------|:--------------------------|:---------------------------------|:-------------------------------- |
| **`1:SK`**   | `783.99 ns` (✅ **1.00x**) | `455.76 ns` (✅ **1.72x faster**) | `1.39 us` (❌ *1.77x slower*)     |
| **`10:SK`**  | `1.80 us` (✅ **1.00x**)   | `N/A`                            | `2.49 us` (❌ *1.38x slower*)     |
| **`50:SK`**  | `9.23 us` (✅ **1.00x**)   | `N/A`                            | `14.72 us` (❌ *1.60x slower*)    |
| **`100:SK`** | `20.74 us` (✅ **1.00x**)  | `N/A`                            | `34.11 us` (❌ *1.65x slower*)    |

### Select Range Secondary Key

|                           | `Native_db`             | `Sqlite`                          |
|:--------------------------|:------------------------|:--------------------------------- |
| **`1:SK value range`**    | `1.48 ms` (✅ **1.00x**) | `671.05 us` (🚀 **2.21x faster**)  |
| **`10:SK value range`**   | `1.61 ms` (✅ **1.00x**) | `1.03 ms` (✅ **1.56x faster**)    |
| **`50:SK value range`**   | `2.89 ms` (✅ **1.00x**) | `3.63 ms` (❌ *1.25x slower*)      |
| **`100:SK value range`**  | `4.17 ms` (✅ **1.00x**) | `6.88 ms` (❌ *1.65x slower*)      |
| **`1:SK random range`**   | `1.66 ms` (✅ **1.00x**) | `725.59 us` (🚀 **2.29x faster**)  |
| **`10:SK random range`**  | `1.78 ms` (✅ **1.00x**) | `1.10 ms` (✅ **1.62x faster**)    |
| **`50:SK random range`**  | `3.45 ms` (✅ **1.00x**) | `3.85 ms` (❌ *1.12x slower*)      |
| **`100:SK random range`** | `4.45 ms` (✅ **1.00x**) | `7.16 ms` (❌ *1.61x slower*)      |

### Delete

:warning: We can see that when all operations are in a single transaction (`n:T`), Native DB has a huge overhead. An issue has been created to resolve this problem https://github.com/vincent-herlemont/native_db/issues/256.

|                       | `Native_db`               | `Redb`                           | `Sqlite`                           |
|:----------------------|:--------------------------|:---------------------------------|:---------------------------------- |
| **`1:SK with n:T`**   | `4.26 us` (✅ **1.00x**)   | `876.24 ns` (🚀 **4.86x faster**) | `813.67 ns` (🚀 **5.24x faster**)   |
| **`1:SK with 1:T`**   | `4.20 ms` (✅ **1.00x**)   | `4.13 ms` (✅ **1.02x faster**)   | `546.21 us` (🚀 **7.70x faster**)   |
| **`10:SK with n:T`**  | `25.64 us` (✅ **1.00x**)  | `N/A`                            | `980.08 ns` (🚀 **26.16x faster**)  |
| **`10:SK with 1:T`**  | `5.01 ms` (✅ **1.00x**)   | `N/A`                            | `576.22 us` (🚀 **8.70x faster**)   |
| **`50:SK with n:T`**  | `133.34 us` (✅ **1.00x**) | `N/A`                            | `1.48 us` (🚀 **90.36x faster**)    |
| **`50:SK with 1:T`**  | `5.82 ms` (✅ **1.00x**)   | `N/A`                            | `538.47 us` (🚀 **10.81x faster**)  |
| **`100:SK with n:T`** | `259.50 us` (✅ **1.00x**) | `N/A`                            | `2.17 us` (🚀 **119.86x faster**)   |
| **`100:SK with 1:T`** | `6.89 ms` (✅ **1.00x**)   | `N/A`                            | `415.64 us` (🚀 **16.58x faster**)  |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

