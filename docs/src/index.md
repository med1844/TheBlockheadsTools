# The Blockheads Tools Docs

Tools for manipulating save files of the mobile game "The Blockheads".

## Installation

This project is highly unstable and we don't have pre-built releases yet. You'll need to compile the tools yourself.

## Installation from source

First, ensure you have the [Rust toolchain](https://rustup.rs/) and [`uv`](https://docs.astral.sh/uv/) installed.

!!! note
    All terminal commands below assume you are running them from the **root of the cloned repository** (e.g., `TheBlockheadsTools/`).

### Python bindings

We recommend using `uv` to manage dependencies. Since the tools are natively written in Rust, they must be compiled before they can be used in Python.

To compile and install the bindings into your active environment, run:

```bash
# Start from the repository root
cd crates/py_bindings
uv sync
# Activate the virtual environment
# e.g. on linux: source .venv/bin/activate
maturin dev --release
```

**Verify Installation:**

To test if the installation was successful, ensure your virtual environment is activated and try executing the following command:

```bash
python -c "import the_blockheads_tools_py"
```

If it silently completes without throwing an error, you're good to go!

### Editor on Web

The editor GUI can run in the browser. To serve it locally, you'll need the WebAssembly target and `trunk`:

```bash
# Add the WebAssembly compilation target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk

# Start from the repository root
cd crates/web_gui

# Serve the web GUI locally
trunk serve --release
```

Then, open your browser and navigate to the address `trunk` provides (usually `localhost:8080`).

## Next steps

- [First steps](getting-started/first-steps.md): Examples of how to use the library to edit save files.
- Setup [emulator](guides/setup-emulator.md) and [server](guides/setup-server.md).
- Game save file structure (Todo)