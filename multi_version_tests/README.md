# Native DB Multi-Version Tests

This is a separate test crate designed to test compatibility between different versions of `native_db` without creating dependency conflicts in the main crate.

## Purpose

This crate allows us to:
- Test compatibility between different versions of `native_db`
- Verify that data formats remain consistent across versions
- Test migration scenarios
- Ensure version isolation works correctly

## Structure

- `native_db_current` - Alias for the current version (from `../`)
- `native_db_v0_8_1` - Alias for version 0.8.1 (from `../../native_db_0_8_x`)

## Running Tests

```bash
# From the main project directory
cd multi_version_tests
cargo test

# Run specific test
cargo test test_version_isolation

# Run with output
cargo test -- --nocapture
```

## Available Tests

- `test_current_version_operations` - Tests basic operations with the current version
- `test_v081_operations` - Tests basic operations with version 0.8.1
- `test_version_isolation` - Tests that both versions can coexist independently

## Adding New Tests

When adding new multi-version tests:

1. Define separate models for each version using the appropriate macro:
   ```rust
   // For current version
   #[native_db_current::native_db]
   struct CurrentModel { ... }
   
   // For v0.8.1
   #[native_db_v0_8_1::native_db]
   struct V081Model { ... }
   ```

2. Use aliased imports:
   ```rust
   use native_db_current::{Builder as CurrentBuilder, Models as CurrentModels};
   use native_db_v0_8_1::{Builder as V081Builder, Models as V081Models};
   ```

3. Test scenarios like:
   - Data format compatibility
   - Schema evolution
   - Migration paths
   - Performance comparisons

## Dependencies

This crate maintains its own set of dependencies to avoid conflicts with the main crate. The dependency versions are chosen to be compatible with the versions being tested. 