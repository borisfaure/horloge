#!/usr/bin/env bash
# Script for running check on your rust projects.
set -e
set -x
set -u

declare -A LANGUAGES
LANGUAGES=(
    [0]="french"
    [1]="english"
)
declare -A EXAMPLES
EXAMPLES=(
    [0]="rgb_leds"
)


run_doc() {
    rustup component add rust-docs
    for EXAMPLE in "${EXAMPLES[@]}"
    do
        cargo doc --example "$EXAMPLE"
    done
    for LANGUAGE in "${LANGUAGES[@]}"
    do
        cargo doc --no-default-features --features "$LANGUAGE"
    done
}

run_fmt() {
    rustup component add rustfmt
    cargo fmt --check
}

run_clippy() {
    rustup component add clippy-preview
    for EXAMPLE in "${EXAMPLES[@]}"
    do
        cargo clippy --example "$EXAMPLE" -- -D warnings
    done
    for LANGUAGE in "${LANGUAGES[@]}"
    do
        cargo clippy --no-default-features --features "$LANGUAGE" -- -D warnings
    done
}

run_check() {
    for EXAMPLE in "${EXAMPLES[@]}"
    do
        cargo check --example "$EXAMPLE"
    done
    for LANGUAGE in "${LANGUAGES[@]}"
    do
        cargo check --no-default-features --features "$LANGUAGE"
    done
}

run_test() {
    cargo test -p utils --target "x86_64-unknown-linux-gnu"
}

run_build() {
    cargo install flip-link
    for EXAMPLE in "${EXAMPLES[@]}"
    do
        cargo build --example "$EXAMPLE"
    done
    for LANGUAGE in "${LANGUAGES[@]}"
    do
        cargo build --no-default-features --features "$LANGUAGE"
    done
}

run_build_release() {
    cargo install flip-link
    for EXAMPLE in "${EXAMPLES[@]}"
    do
        cargo build --release --example "$EXAMPLE"
    done
    for LANGUAGE in "${LANGUAGES[@]}"
    do
        cargo build --release --no-default-features --features "$LANGUAGE"
    done
}

case $1 in
    doc)
        run_doc
        ;;
    fmt)
        run_fmt
        ;;
    check)
        run_check
        ;;
    clippy)
        run_clippy
        ;;
    test)
        run_test
        ;;
    build)
        run_build
        ;;
    build-release)
        run_build_release
        ;;
esac
