#!/usr/bin/env bash

# How to test:
# - Use docker `docker run -it --rm -v $(pwd):/mnt/native_db rust:bullseye bash` 
# - `cd /mnt/native_db`
# - `export CARGO_TOKEN=<your_cargo_token>`
# - `./cargo_publish.sh`

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

set -e
set -x

ARG_TOKEN="--token=$CARGO_TOKEN"

cd $DIR/native_db_macro

# Temporarily disable 'set -e' to handle the error manually
set +e
OUTPUT=$(cargo publish $ARG_TOKEN "$@" 2>&1)
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -ne 0 ]; then
  if echo "$OUTPUT" | grep -q "crate version .* is already uploaded"; then
    echo "Warning: $OUTPUT"
  else
    echo "Error: $OUTPUT"
    exit $EXIT_CODE
  fi
fi

cd $DIR
cargo publish $ARG_TOKEN $@