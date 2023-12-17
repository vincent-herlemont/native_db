#!/bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Bash script to update version for native_db and native_db_macro

# Semantic release version obtained from argument
NEW_VERSION=$1

# Exit if NEW_VERSION is not set
if [ -z "$NEW_VERSION" ]; then
  echo "NEW_VERSION argument not set"
  exit 1
fi

# Directories containing Cargo.toml files to update
declare -a directories=("." "native_db_macro")

for directory in "${directories[@]}"
do
  # Check if Cargo.toml exists in the directory
  if [ -f "$directory/Cargo.toml" ]; then
    echo "Updating version in $directory/Cargo.toml to $NEW_VERSION"
    # Use sed to find and replace the version string in the Cargo.toml
    sed -i -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"$NEW_VERSION\"/g" "$directory/Cargo.toml"
    # Use sed to find and replace the version string in the README.md
    sed -i -E "s/native_db = \"[0-9]+\.[0-9]+\.[0-9]+\"/native_db = \"$NEW_VERSION\"/g" "$directory/README.md"

    # Update the dependency version for native_db_macro in native_db's Cargo.toml
    if [ "$directory" == "." ]; then
      sed -i -E "s/native_db_macro = \{ version = \"[0-9]+\.[0-9]+\.[0-9]+\", path = \"native_db_macro\" \}/native_db_macro = { version = \"$NEW_VERSION\", path = \"native_db_macro\" }/g" "$directory/Cargo.toml"
    fi
  fi
done


cd "$DIR/"

# Commit
git commit --all --message "chore: update version to $NEW_VERSION"
git push