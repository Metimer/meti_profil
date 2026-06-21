# Environment Setup

This project uses a Rust workspace together with a Python virtual environment and maturin.

## Create the environment

1. Install Rust via rustup if it is not already available:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
   ```

2. Source the cargo env so `rustc`, `cargo`, and other Rust tools are available:

   ```bash
   source "$HOME/.cargo/env"
   ```

3. Create a Python virtual environment:

   ```bash
   python3 -m venv .venv
   ```

4. Activate the Python virtual environment:

   ```bash
   source .venv/bin/activate
   ```

5. Install the development dependencies:

   ```bash
   pip install maturin pytest pandas polars pyarrow
   ```

## Verify tools

After activation, confirm the core tools are on your PATH:

```bash
rustc --version
cargo --version
maturin --version
```

## Project workflow

> The commands below become available after the workspace is bootstrapped in Task 1.

### Build / develop the Python package

```bash
maturin develop
```

### Run tests

Run the Rust workspace tests:

```bash
cargo test --workspace
```

Run the Python tests:

```bash
pytest tests/python -v
```
