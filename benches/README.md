# Benchmarks

## Table of Contents

- [Overview](#overview)
- [Benchmark Results](#benchmark-results)
    - [Insert](#insert)
    - [Get](#get)
    - [Select Range Secondary Key](#select-range-secondary-key)
    - [Delete](#delete)

## Overview


- :warning: This benchmark is an initial version and can certainly be greatly improved to make the results as relevant as possible. Feel free to open issues to improve it.
- :point_right: Native DB will be further improved in the future, as performance issues have not yet been addressed. That is indeed the purpose of this benchmark: to provide visibility on what needs to be improved.

Comparison between [`Native DB`](https://github.com/vincent-herlemont/native_db) vs [`Redb`](https://github.com/cberner/redb) vs [`SQLite`](https://www.sqlite.org/)

- Why compare with `Redb`?
  - To highlight the `Native DB` overhead. Because `Redb` is the backend of `Native DB`, it should normally always be faster than `Native DB`.
- Why compare with `SQLite`?
  - Because even though `SQLite` offers many more options, `Native DB` can be seen as a very light alternative.
- And other databases?
  - Knowing the capabilities of `Native DB` compared to `Redb` with the benchmark below, you can check the `Redb` benchmark here: [cberner/redb/benchmarks](https://github.com/cberner/redb?tab=readme-ov-file#benchmarks)

The benchmarks ignore the following:
 - [`native_model`](https://github.com/vincent-herlemont/native_model) overhead.
 - Serialization overhead used by `native_model`, such as `bincode`, `postcard`, etc.
 - The fact that `Redb` can write data using zero-copy.

Explanation:
 - `1:SK`, `10:SK`, `50:SK`, `100:SK`, `N:SK`: in this case, `N` is the number of secondary keys (`SK`) for the same data. Regarding `SQLite`, it is the column, with each having a secondary index.
 - `1:T`, `n:T` represent the number of operations per transaction.
   - `1:T` means one operation per transaction. For example, for insertion, it means there is only one insert operation per transaction.
   - `n:T` means `n` operations per transaction, where `n` is defined by `criterion`, meaning that all operations are within a single transaction.
 - We can see that `Redb` sometimes has no comparisons (`N/A`) because `Redb` is a key-value database and does not support secondary indexes. Therefore, it is pointless to compare it with more or fewer secondary indexes.

## Benchmark Results

### Insert

|                       | `Native_db`               | `Native_db_twophasecommit`          | `Native_db_quickrepair`          | `Redb`                          | `Sqlite`                         |
|:----------------------|:--------------------------|:------------------------------------|:---------------------------------|:--------------------------------|:-------------------------------- |
| **`1:SK with n:T`**   | `5.36 us` (✅ **1.00x**)   | `5.42 us` (✅ **1.01x slower**)      | `5.37 us` (✅ **1.00x slower**)   | `1.09 us` (🚀 **4.90x faster**)  | `1.68 us` (🚀 **3.19x faster**)   |
| **`1:SK with 1:T`**   | `33.43 us` (✅ **1.00x**)  | `428.91 us` (❌ *12.83x slower*)     | `33.31 us` (✅ **1.00x faster**)  | `15.66 us` (🚀 **2.13x faster**) | `47.86 us` (❌ *1.43x slower*)    |
| **`10:SK with n:T`**  | `31.21 us` (✅ **1.00x**)  | `31.36 us` (✅ **1.00x slower**)     | `31.08 us` (✅ **1.00x faster**)  | `N/A`                           | `3.38 us` (🚀 **9.25x faster**)   |
| **`10:SK with 1:T`**  | `136.49 us` (✅ **1.00x**) | `525.33 us` (❌ *3.85x slower*)      | `139.00 us` (✅ **1.02x slower**) | `N/A`                           | `50.10 us` (🚀 **2.72x faster**)  |
| **`50:SK with n:T`**  | `149.93 us` (✅ **1.00x**) | `149.58 us` (✅ **1.00x faster**)    | `148.39 us` (✅ **1.01x faster**) | `N/A`                           | `16.35 us` (🚀 **9.17x faster**)  |
| **`50:SK with 1:T`**  | `537.67 us` (✅ **1.00x**) | `899.42 us` (❌ *1.67x slower*)      | `539.79 us` (✅ **1.00x slower**) | `N/A`                           | `66.48 us` (🚀 **8.09x faster**)  |
| **`100:SK with n:T`** | `294.09 us` (✅ **1.00x**) | `291.53 us` (✅ **1.01x faster**)    | `291.70 us` (✅ **1.01x faster**) | `N/A`                           | `44.57 us` (🚀 **6.60x faster**)  |
| **`100:SK with 1:T`** | `916.36 us` (✅ **1.00x**) | `1.23 ms` (❌ *1.34x slower*)        | `923.44 us` (✅ **1.01x slower**) | `N/A`                           | `98.20 us` (🚀 **9.33x faster**)  |

### Get

|              | `Native_db`               | `Native_db_twophasecommit`          | `Native_db_quickrepair`          | `Redb`                           | `Sqlite`                         |
|:-------------|:--------------------------|:------------------------------------|:---------------------------------|:---------------------------------|:-------------------------------- |
| **`1:SK`**   | `960.70 ns` (✅ **1.00x**) | `960.34 ns` (✅ **1.00x faster**)    | `957.75 ns` (✅ **1.00x faster**) | `492.15 ns` (🚀 **1.95x faster**) | `1.99 us` (❌ *2.07x slower*)     |
| **`10:SK`**  | `2.49 us` (✅ **1.00x**)   | `2.50 us` (✅ **1.00x slower**)      | `2.50 us` (✅ **1.01x slower**)   | `N/A`                            | `3.34 us` (❌ *1.34x slower*)     |
| **`50:SK`**  | `113.49 us` (✅ **1.00x**) | `112.27 us` (✅ **1.01x faster**)    | `114.37 us` (✅ **1.01x slower**) | `N/A`                            | `21.13 us` (🚀 **5.37x faster**)  |
| **`100:SK`** | `241.06 us` (✅ **1.00x**) | `265.73 us` (✅ **1.10x slower**)    | `246.91 us` (✅ **1.02x slower**) | `N/A`                            | `49.21 us` (🚀 **4.90x faster**)  |

### Select Range Secondary Key

|                           | `Native_db`             | `Native_db_twophasecommit`          | `Native_db_quickrepair`          | `Sqlite`                          |
|:--------------------------|:------------------------|:------------------------------------|:---------------------------------|:--------------------------------- |
| **`1:SK value range`**    | `1.88 ms` (✅ **1.00x**) | `1.88 ms` (✅ **1.00x faster**)      | `1.91 ms` (✅ **1.02x slower**)   | `706.00 us` (🚀 **2.66x faster**)  |
| **`10:SK value range`**   | `2.10 ms` (✅ **1.00x**) | `2.06 ms` (✅ **1.02x faster**)      | `2.06 ms` (✅ **1.02x faster**)   | `1.35 ms` (✅ **1.55x faster**)    |
| **`50:SK value range`**   | `5.02 ms` (✅ **1.00x**) | `4.65 ms` (✅ **1.08x faster**)      | `4.66 ms` (✅ **1.08x faster**)   | `4.64 ms` (✅ **1.08x faster**)    |
| **`100:SK value range`**  | `6.99 ms` (✅ **1.00x**) | `7.02 ms` (✅ **1.00x slower**)      | `6.92 ms` (✅ **1.01x faster**)   | `8.55 ms` (❌ *1.22x slower*)      |
| **`1:SK random range`**   | `1.90 ms` (✅ **1.00x**) | `1.87 ms` (✅ **1.01x faster**)      | `1.92 ms` (✅ **1.01x slower**)   | `758.63 us` (🚀 **2.50x faster**)  |
| **`10:SK random range`**  | `2.15 ms` (✅ **1.00x**) | `2.06 ms` (✅ **1.04x faster**)      | `2.10 ms` (✅ **1.02x faster**)   | `1.37 ms` (✅ **1.56x faster**)    |
| **`50:SK random range`**  | `4.43 ms` (✅ **1.00x**) | `4.32 ms` (✅ **1.03x faster**)      | `4.30 ms` (✅ **1.03x faster**)   | `4.89 ms` (✅ **1.10x slower**)    |
| **`100:SK random range`** | `5.47 ms` (✅ **1.00x**) | `8.25 ms` (❌ *1.51x slower*)        | `6.21 ms` (❌ *1.14x slower*)     | `9.80 ms` (❌ *1.79x slower*)      |

### Delete

:warning: We can see that when all operations are in a single transaction (`n:T`), `Native DB` has a huge overhead. An issue has been created to resolve this problem: [#256](https://github.com/vincent-herlemont/native_db/issues/256).

|                       | `Native_db`               | `Native_db_twophasecommit`          | `Native_db_quickrepair`          | `Redb`                          | `Sqlite`                          |
|:----------------------|:--------------------------|:------------------------------------|:---------------------------------|:--------------------------------|:--------------------------------- |
| **`1:SK with n:T`**   | `6.00 us` (✅ **1.00x**)   | `5.99 us` (✅ **1.00x faster**)      | `6.07 us` (✅ **1.01x slower**)   | `1.15 us` (🚀 **5.22x faster**)  | `1.23 us` (🚀 **4.90x faster**)    |
| **`1:SK with 1:T`**   | `30.76 us` (✅ **1.00x**)  | `30.75 us` (✅ **1.00x faster**)     | `31.38 us` (✅ **1.02x slower**)  | `14.65 us` (🚀 **2.10x faster**) | `46.96 us` (❌ *1.53x slower*)     |
| **`10:SK with n:T`**  | `36.93 us` (✅ **1.00x**)  | `37.24 us` (✅ **1.01x slower**)     | `37.18 us` (✅ **1.01x slower**)  | `N/A`                           | `1.39 us` (🚀 **26.60x faster**)   |
| **`10:SK with 1:T`**  | `129.28 us` (✅ **1.00x**) | `127.75 us` (✅ **1.01x faster**)    | `131.43 us` (✅ **1.02x slower**) | `N/A`                           | `47.85 us` (🚀 **2.70x faster**)   |
| **`50:SK with n:T`**  | `176.44 us` (✅ **1.00x**) | `174.57 us` (✅ **1.01x faster**)    | `177.47 us` (✅ **1.01x slower**) | `N/A`                           | `1.79 us` (🚀 **98.42x faster**)   |
| **`50:SK with 1:T`**  | `501.98 us` (✅ **1.00x**) | `501.80 us` (✅ **1.00x faster**)    | `501.82 us` (✅ **1.00x faster**) | `N/A`                           | `51.22 us` (🚀 **9.80x faster**)   |
| **`100:SK with n:T`** | `349.38 us` (✅ **1.00x**) | `350.17 us` (✅ **1.00x slower**)    | `349.73 us` (✅ **1.00x slower**) | `N/A`                           | `2.36 us` (🚀 **148.25x faster**)  |
| **`100:SK with 1:T`** | `862.53 us` (✅ **1.00x**) | `841.21 us` (✅ **1.03x faster**)    | `851.42 us` (✅ **1.01x faster**) | `N/A`                           | `54.53 us` (🚀 **15.82x faster**)  |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)
