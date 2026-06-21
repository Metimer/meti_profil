# Environment Setup

This project uses a Rust workspace together with a Python virtual environment and maturin. Before working on it, activate the required environment.

## Activate the environment

1. Source cargo env so `rustc`, `cargo`, and other Rust tools are available:

   ```bash
   source "$HOME/.cargo/env"
   ```

2. Activate the Python virtual environment:

   ```bash
   source .venv/bin/activate
   ```

## Verify tools

After activation, confirm the core tools are on your PATH:

```bash
rustc --version
cargo --version
maturin --version
```

## Build / develop the Python package

```bash
maturin develop
```

## Run tests

Run the Rust workspace tests:

```bash
cargo test --workspace
```

Run the Python tests:

```bash
pytest tests/python -v
```
