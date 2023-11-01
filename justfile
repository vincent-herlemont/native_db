set shell := ["nu", "-c"]

default:
    @just --list --unsorted;

build_no_default:
    cargo build --no-default-features

build_default:
    cargo build

build_all: build_no_default build_default

test_no_default:
    cargo test --no-default-features

test_default:
    cargo test

test_all: test_no_default test_default


expand test_file_name:
    cargo expand --test {{test_file_name}} | save --raw {{test_file_name}}.expanded.rs