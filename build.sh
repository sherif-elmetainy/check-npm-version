#!/usr/bin/env bash

set -e
SCRIPT_DIR=$(realpath "$(dirname "${BASH_SOURCE[0]}")")

pushd "${SCRIPT_DIR}" > /dev/null

cargo build --release

cp ./target/release/check-npm-version ~/.local/bin/

popd > /dev/null