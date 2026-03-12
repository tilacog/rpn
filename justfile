# List available recipes
default:
    @just --list

# Format all Rust source files
fmt:
    cargo fmt

# Run clippy with warnings denied
clippy:
    cargo clippy -- -D warnings

# Run all tests
test:
    cargo test

# Build the project
build:
    cargo build

# Run all checks: fmt, clippy, test, build
check:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test
    cargo build

# Run every step: fmt, clippy, test, build
ci: fmt clippy test build
