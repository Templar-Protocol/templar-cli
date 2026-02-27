# Templar CLI — Build & Test Commands

# Run all tests
test:
    cargo nextest run

# Run tests with coverage report
cov:
    cargo llvm-cov nextest --html

# Check formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Build release binary
build:
    cargo build --release

# Generate Rust API docs
doc:
    cargo doc --no-deps --document-private-items

# Build mdbook user guide
book:
    mdbook build docs/

# Full CI check: fmt + lint + test + doc
ci: fmt-check lint test doc book

# Clean build artifacts
clean:
    cargo clean
