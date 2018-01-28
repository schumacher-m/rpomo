#!/usr/bin/env bash
cargo bump
cargo build --release
cargo install --force
