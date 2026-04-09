# Justfile for redis-bridge project
# https://just.systems

# Default target
default:
    just build

# Build the project in release mode
build:
    cargo build --release

# Check the project without building
check:
    cargo check --all-targets --all-features

# Run clippy with pedantic lints
clippy:
    RUSTFLAGS="-W clippy::pedantic" cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test --all-targets --all-features

# Run tests with LLVM coverage report
# Requires: cargo-llvm-cov (cargo install cargo-llvm-cov)
coverage:
    cargo llvm-cov --all-targets --all-features --lcov --output-path lcov.info
    cargo llvm-cov report --html

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt --all

# Format check
fmt-check:
    cargo fmt --all -- --check

# Full CI-style check: fmt, check, clippy, test
ci: fmt-check check clippy test
