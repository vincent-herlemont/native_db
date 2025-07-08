# Native DB Release Documentation

This document provides comprehensive instructions for performing releases of the native_db project, including both automated and manual release processes.

## Table of Contents

1. [Overview](#overview)
2. [Automated Release Process](#automated-release-process)
3. [Manual Release Process](#manual-release-process)
4. [Version Management](#version-management)
5. [Branch Strategy](#branch-strategy)
6. [Troubleshooting](#troubleshooting)

## Overview

Native DB uses [semantic-release](https://semantic-release.gitbook.io/semantic-release/) for automated version management and package publishing. The release process includes:

- Automatic version determination based on commit messages
- Version updates across multiple files
- Publishing to crates.io
- GitHub release creation
- Support for both main and maintenance branches

### Key Components

- **GitHub Actions Workflow**: `.github/workflows/release.yml`
- **Semantic Release Config**: `.releaserc`
- **Version Update Script**: `version_update.sh`
- **Cargo Publish Script**: `cargo_publish.sh`
- **Packages**: `native_db` and `native_db_macro`

## Automated Release Process

### Triggering Automated Releases

Automated releases are triggered in two ways:

1. **Push to main or 0.8.x branch** (automatic)
   - Runs in dry-run mode
   - Analyzes commits but doesn't publish

2. **Manual workflow dispatch** (publish mode)
   - Actually publishes the release
   - Can be triggered from GitHub Actions UI

### How It Works

1. **Commit Analysis**: Semantic-release analyzes commit messages to determine the next version
2. **Version Update**: Runs `version_update.sh` to update versions in:
   - `Cargo.toml` files
   - `README.md`
   - `src/metadata/current_version.rs`
   - `src/metadata/current_native_model_version.rs`
   - Test files
3. **Git Commit**: Creates a version bump commit
4. **Publishing**: Runs `cargo_publish.sh` to publish to crates.io
5. **GitHub Release**: Creates a GitHub release with changelog

### Commit Message Format

The project follows conventional commits with these release rules:

- **Breaking changes** → Minor version bump (e.g., 0.8.0 → 0.9.0)
- **feat:** → Minor version bump
- **fix:**, **perf:**, **docs:** → Patch version bump
- **BREAKING CHANGE:** in footer → Minor version bump

## Manual Release Process

### Prerequisites

1. **Rust and Cargo** installed
2. **Crates.io token**: Set as `CARGO_TOKEN` environment variable
3. **Git push permissions** to the repository
4. **GitHub Personal Access Token** (for creating releases)

### Step-by-Step Manual Release

#### 1. Determine Next Version

Check the current version and analyze commits since last release:

```bash
# Current version
grep "^version" Cargo.toml

# Commits since last tag
git log $(git describe --tags --abbrev=0)..HEAD --oneline

# Analyze commit types
git log $(git describe --tags --abbrev=0)..HEAD --pretty=format:"%s" | grep -E "^(feat|fix|docs|perf|BREAKING CHANGE):"
```

Based on conventional commits:
- Breaking changes or feat: Increment minor version (0.8.1 → 0.9.0)
- fix, docs, perf: Increment patch version (0.8.1 → 0.8.2)

#### 2. Update Version Numbers

Run the version update script:

```bash
./version_update.sh <new_version>
# Example: ./version_update.sh 0.8.2
```

This script updates:
- `./Cargo.toml` - Main package version
- `./native_db_macro/Cargo.toml` - Macro package version
- `./README.md` - Documentation version references
- `./src/metadata/current_version.rs` - Runtime version constant
- `./src/metadata/current_native_model_version.rs` - Native model version
- Test files with version assertions

The script automatically commits and pushes these changes.

#### 3. Create and Push Tag

```bash
# Create annotated tag
git tag -a <version> -m "Release <version>"
# Example: git tag -a 0.8.2 -m "Release 0.8.2"

# Push tag
git push origin <version>
```

#### 4. Publish to Crates.io

Set your cargo token and run the publish script:

```bash
# Set token (get from https://crates.io/me)
export CARGO_TOKEN="your-token-here"

# Run publish script
./cargo_publish.sh
```

The script:
1. Publishes `native_db_macro` first (dependency)
2. Handles "already published" errors gracefully
3. Publishes `native_db` main package

#### 5. Create GitHub Release

1. Go to https://github.com/vincent-herlemont/native_db/releases/new
2. Select the tag you just created
3. Set release title: `v<version>` (e.g., `v0.8.2`)
4. Generate release notes from commits
5. Publish release

### Manual Release Checklist

- [ ] Analyze commits and determine version bump type
- [ ] Run `./version_update.sh <new_version>`
- [ ] Review the automated commit
- [ ] Create and push version tag
- [ ] Set `CARGO_TOKEN` environment variable
- [ ] Run `./cargo_publish.sh`
- [ ] Create GitHub release
- [ ] Verify packages on crates.io

## Version Management

### Version Locations

The project maintains version numbers in multiple locations:

1. **Cargo.toml files**:
   - `./Cargo.toml` - Main package
   - `./native_db_macro/Cargo.toml` - Macro package
   - Dependency reference in main Cargo.toml

2. **Source files**:
   - `src/metadata/current_version.rs` - Runtime version constant
   - `src/metadata/current_native_model_version.rs` - Native model version

3. **Documentation**:
   - `README.md` - Installation instructions

4. **Tests**:
   - `tests/metadata/current_version.rs` - Version assertions

### Native Model Version

The project depends on `native_model` for serialization. The version update script:
1. Extracts the current native_model version from Cargo.toml
2. Updates references in README.md
3. Updates the version constant in source files
4. Updates test assertions

## Branch Strategy

### Main Branch (`main`)

- Primary development branch
- Receives all new features
- Releases follow semantic versioning
- Currently configured for automated releases

### Maintenance Branch (`0.8.x`)

- For patch releases to 0.8.x series
- Cherry-pick fixes from main
- Currently **temporarily disabled** in semantic-release
- Will be re-enabled after 0.9.0 release on main

### Re-enabling 0.8.x Branch Releases

After version 0.9.0 is released on main, update `.releaserc`:

```json
"branches": [
  "main",
  {
    "name": "0.8.x"
  }
],
```

## Troubleshooting

### Common Issues

#### 1. Semantic Release Version Conflict

**Error**: "Version range conflict when both branches have same version"

**Solution**: Temporarily disable maintenance branch in `.releaserc` until main branch advances.

#### 2. Cargo Publish Already Uploaded

**Error**: "crate version X.X.X is already uploaded"

**Solution**: The `cargo_publish.sh` script handles this gracefully. It's usually safe to ignore if doing a retry.

#### 3. Version Update Script Fails

**Issue**: Script can't find files or sed commands fail

**Solutions**:
- Ensure you're in the project root directory
- Check file permissions
- Verify sed syntax works on your platform (Linux/macOS differences)

#### 4. GitHub Actions Release Fails

**Common causes**:
- Token permissions insufficient
- Branch protection rules blocking pushes
- Concurrent workflow runs

**Solutions**:
- Check `PAT_GLOBAL` secret has write permissions
- Ensure branch allows GitHub Actions to push
- Wait for other workflows to complete

### Testing Release Process

#### Using Docker

Test version update script:
```bash
docker run -it --rm -v $(pwd):/mnt/native_db ubuntu bash
cd /mnt/native_db
./version_update.sh 0.8.0
```

Test cargo publish:
```bash
docker run -it --rm -v $(pwd):/mnt/native_db rust:bullseye bash
cd /mnt/native_db
export CARGO_TOKEN=<your_token>
./cargo_publish.sh --dry-run
```

### Manual Rollback

If a release needs to be yanked:

1. Yank from crates.io:
   ```bash
   cargo yank --version <version> native_db
   cargo yank --version <version> native_db_macro
   ```

2. Delete GitHub release (optional)

3. Revert version bump commit:
   ```bash
   git revert <commit-hash>
   git push
   ```

## Best Practices

1. **Always test locally first** using dry-run or Docker
2. **Review automated commits** before publishing
3. **Keep commit messages clean** - they determine versions
4. **Document breaking changes** in commit messages
5. **Coordinate releases** between main and maintenance branches
6. **Monitor CI status** before and after releases
7. **Verify published packages** on crates.io after release

## Additional Resources

- [Semantic Release Documentation](https://semantic-release.gitbook.io/semantic-release/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Cargo Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)