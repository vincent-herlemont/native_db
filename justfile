set shell := ["nu", "-c"]

default:
    @just --list --unsorted;

build_no_default:
    cargo build --no-default-features

build_default:
    cargo build

build_with_optional:
    cargo build -F chrono -F uuid -F tokio

build_all: build_no_default build_default build_with_optional

test_no_default:
    cargo test --no-default-features

test_default:
    cargo test

test_with_optional:
    cargo test -F chrono -F uuid -F tokio

test_all: test_no_default test_default test_with_optional


bench_build:
    cargo bench --no-run

bench:
    CRITERION_DEBUG=1 cargo bench; \
    start ./target/criterion/report/index.html

expand test_file_name:
    rm -f {{test_file_name}}.expanded.rs; \
    cargo expand --test {{test_file_name}} | save --raw {{test_file_name}}.expanded.rs

serve_docs:
    cd book; mdbook serve --open

bot_collect_rust_docs:
    #!/usr/bin/env bash
    find . -name '*.rs' -exec grep -h '^\s*///' {} \; | sed 's/^\s*\/\/\/ //' > collected_comments.txt
