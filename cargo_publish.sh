#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

set -e
set -x

ARG_TOKEN="--token=$CARGO_TOKEN"

cd $DIR/struct_db_macro
cargo publish $ARG_TOKEN $@

cd $DIR
cargo publish $ARG_TOKEN $@