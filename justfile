#!/usr/bin/env just --justfile
alias b := build

dev := '1'

# automatically build on each change
autobuild:
    cargo watch -x build

# run benchmarks
bench:
    cargo bench

# build release binary
build:
    cargo build

# rebuild docs
doc:
    cargo doc

# rebuild docs and start simple static server
docs +PORT='40000':
    cargo doc && http target/doc -p {{PORT}}

# start server for docs and update upon changes
docslive:
    light-server -c .lightrc

# rebuild docs and start simple static server that watches for changes (in parallel)
docw +PORT='40000':
    parallel --lb ::: "cargo watch -x doc" "http target/doc -p {{PORT}}"

fix:
    cargo fix

test:
    cargo test
