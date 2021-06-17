#!/bin/sh

set -e

for DEMO in $(ls -d */)
do
	cargo build --manifest-path "${DEMO}Cargo.toml"
done
