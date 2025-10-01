#!/usr/bin/env bash

set -e
SCRIPT_DIR=$(realpath "$(dirname "${BASH_SOURCE[0]}")")

pushd "${SCRIPT_DIR}" > /dev/null

cargo build --release

if [[ "$(uname -o)" == "Msys" ]]; then
  cp ./target/release/check-npm-version.exe ~/.local/
else
  cp ./target/release/check-npm-version ~/.local/bin/
fi

popd > /dev/null