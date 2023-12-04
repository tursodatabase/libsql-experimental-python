# libSQL API for Python

[![PyPI](https://badge.fury.io/py/libsql-experimental.svg)](https://badge.fury.io/py/libsql-experimental)

libSQL is an open source, open contribution fork of SQLite. We aim to evolve it to suit many more use cases than SQLite was originally designed for.

This source repository contains libSQL API bindings for Python, which aim to be compatible with the [sqlite3](https://docs.python.org/3/library/sqlite3.html) module.

## Install

You can install the current release _(MacOS and Linux)_:

```
$ pip install libsql-experimental
```

## Documentation

* [API reference](docs/api.md)

## Getting Started

To try out your first libsql program, start the Python interpreter:

```shell
$ python
```

and then:

```python
>>> import libsql_experimental as libsql
>>> con = libsql.connect("hello.db")
>>> cur = con.cursor()
>>> cur.execute("CREATE TABLE users (id INTEGER, email TEXT);")
<builtins.Result object at 0x102dcf8d0>
>>> cur.execute("INSERT INTO users VALUES (1, 'alice@example.org')")
<builtins.Result object at 0x102dcf4b0>
>>> cur.execute("SELECT * FROM users").fetchone()
(1, 'alice@example.org')
```

#### Connecting to a database

```python
import libsql_experimental as libsql

con = libsql.connect("hello.db")
cur = con.cursor()
```

#### Embedded replica

```python
import libsql_experimental as libsql

url = os.getenv("LIBSQL_URL")
auth_token = os.getenv("LIBSQL_AUTH_TOKEN")

con = libsql.connect("hello.db", sync_url=url, auth_token=auth_token)
con.sync()
```

#### Creating a table

```python
cur.execute("CREATE TABLE users (id INTEGER, email TEXT);")
```

#### Inserting rows into a table

```python
cur.execute("INSERT INTO users VALUES (1, 'alice@example.org')")
```

#### Querying rows from a table

```python
print(cur.execute("SELECT * FROM users").fetchone())
```

## Developing

Setup the development environment:

```sh
python3 -m venv .env
source .env/bin/activate
pip3 install maturin pyperf pytest
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

## License

This project is licensed under the [MIT license].

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in libSQL by you, shall be licensed as MIT, without any additional
terms or conditions.

[MIT license]: https://github.com/libsql/libsql-experimental-python/blob/main/LICENSE.md
