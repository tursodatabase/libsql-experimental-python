# Local

This example demonstrates how to create and use an encrypted SQLite database with libSQL.

## Install Dependencies

```bash
pip install libsql
```

## Running

Execute the example:

```bash
python3 main.py
```

This example will:

1. Create a new table called `users`.
2. Start a transaction.
3. Insert multiple users within the transaction.
4. Demonstrate how to rollback a transaction.
5. Start another transaction.
6. Insert more users and commit the transaction.
7. Query and display the final state of the `users` table.
