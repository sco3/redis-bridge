# Justfile for redis-bridge project
# https://just.systems

# Default target: show available targets
default:
    just --list

# Build the project in release mode
build:
    cargo build --release

# Check the project without building
check:
    cargo check --all-targets --all-features

# Run clippy with pedantic lints
clippy:
    RUSTFLAGS="-W clippy::pedantic" cargo clippy --all-targets --all-features -- -D warnings

# Alias for clippy
lint: clippy

# Run tests
test:
    cargo test --all-targets --all-features

# Install LLVM coverage tool
install-coverage:
    cargo install cargo-llvm-cov

# Show coverage text summary (excludes bin/ and main.rs)
coverage: install-coverage
    cargo llvm-cov --all-targets --all-features --ignore-filename-regex "(src/bin/.*|src/main\.rs)" --summary-only

# Generate HTML coverage report and print the URL
coverage-html: install-coverage
    cargo llvm-cov --all-targets --all-features --ignore-filename-regex "(src/bin/.*|src/main\.rs)" --html
    @echo ""
    @echo "📊 Coverage report generated (src/bin/ and src/main.rs excluded):"
    @echo "   file://$(pwd)/target/llvm-cov/html/index.html"

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

# Run end-to-end smoke test (requires running Redis + gateway)
smoke-test:
    cargo run --bin smoke-test

# Run end-to-end smoke test in release mode
smoke:
    cargo run --bin smoke-test --release
