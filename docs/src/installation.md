# Installation

## From Source

```bash
# Clone the repository
git clone https://github.com/Templar-Protocol/templar-cli.git
cd templar-cli

# Build release binary
cargo build --release

# The binary is at target/release/templar
# Optionally install to your PATH:
cargo install --path .
```

## Requirements

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- For development: `cargo-nextest`, `mdbook`, `just`

```bash
cargo install cargo-nextest mdbook just
```

## Verify Installation

```bash
templar --version
```
