# Contributing to libSQL Python SDK

## Developing

Setup the development environment:

```sh
python3 -m venv .env
source .env/bin/activate
pip3 install maturin pyperf pytest
```

Or you can use NIX to drop you into a shell with everything installed

```
nix-shell
```

Build the development version and use it:

```
maturin develop && python3 example.py
```

Run the tests:

```sh
pytest
```

Run the libSQL benchmarks:

```sh
python3 perf-libsql.py
```

Run the SQLite benchmarks for comparison:

```sh
python3 perf-sqlite3.py
```
