# Local

This example demonstrates how to use libSQL with a remote database.

## Install Dependencies

```bash
pip install libsql-experimental
```

## Running

Execute the example:

```bash
TURSO_DATABASE_URL="..." TURSO_AUTH_TOKEN="..." python3 main.py
```

This will connect to a remote database, insert some data, and query it.
