## Dev

Ensure you have uv installed before proceeding.

### Set up venv

```fish
uv sync
source .venv/bin/activate.fish
```

### Install `the_blockheads_tools_py`

```fish
maturin dev
```

Add `-r` if you want it to be fast.

### Run python tests

```fish
cd tests
pytest
```
