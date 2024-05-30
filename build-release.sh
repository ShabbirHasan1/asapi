#!/usr/bin/bash
mv Cargo.toml Cargo.toml.dev
mv Cargo.toml.release Cargo.toml

cargo build --release

mv Cargo.toml Cargo.toml.release
mv Cargo.toml.dev Cargo.toml

cp target/release/asapi ./
chmod +x ./asapi
