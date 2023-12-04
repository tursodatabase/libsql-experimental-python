# `libsql`: SQLite compatible interface for libSQL

## Module functions

### connect(database) ⇒ Connection

Creates a new database connection.

| Param    | Type                | Description               |
| -------- | ------------------- | ------------------------- |
| database | <code>string</code> | Path to the database file |

## `Connection` objects

### cursor() ⇒ Cursor

Creates a new database cursor.

### blobopen()

Unimplemented.

### commit()

Commits the current transaction and starts a new one.

### rollback()

Rolls back the current transaction and starts a new one.

### close()

Closes the database connection.

### execute(sql, parameters=())

Create a new cursor object and executes the SQL statement.

### executemany(sql, parameters)

Create a new cursor object and Execute the SQL statement for every item in `parameters` array.

| Param      | Type                | Description                                    |
| ---------- | ------------------- | ---------------------------------------------- |
| sql        | <code>string</code> | Path to the database file                      |
| parameters | <code>array</code>  | Array of parameter tuples to execute SQL with. |

### executescript()

Unimplemented.

### create_function()

Unimplemented.

### create_aggregate()

Unimplemented.

### create_window_function()

Unimplemented.

### create_collation()

Unimplemented.

### interrupt()

Unimplemented.

### set_authorizer()

Unimplemented.

### set_progress_handler()

Unimplemented.

### set_trace_callback()

Unimplemented.

### enable_load_extension()

Unimplemented.

### load_extension()

Unimplemented.

### iterdump()

Unimplemented.

### backup()

Unimplemented.

### getlimit()

Unimplemented.

### setlimit()

Unimplemented.

### getconfig()

Unimplemented.

### setconfig()

Unimplemented.

### serialize()

Unimplemented.

### deserialize()

Unimplemented.

### autocommit

Unimplemented.

### in_transaction

Returns `True` if there's an active transaction with uncommitted changes; otherwise returns `False`.

### isolation_level

Unimplemented.

### row_factory

Unimplemented.

### text_factory 

Unimplemented.

### total_changes

Unimplemented.

## `Cursor` objects

### execute(sql, parameters=())

Execute one SQL statement.

### executemany(sql, parameters)

Execute the SQL statement for every item in `parameters` array.

| Param      | Type                | Description                                    |
| ---------- | ------------------- | ---------------------------------------------- |
| sql        | <code>string</code> | Path to the database file                      |
| parameters | <code>array</code>  | Array of parameter tuples to execute SQL with. |

### executescript()

Unimplemented.

### fetchone()

Return next row in result set.

### fetchmany(size = cursor.arraysize)

Return `size` next rows in result set. If there are no more rows left, returns an empty list.

### fetchall()

Return all rows in result set.

### close()

Unimplemented.

### setinputsizes()

Unimplemented.

### setoutputsize()

Unimplemented.

### arraysize

The number of rows returned by `fetchmany()` by default.

### connection

Unimplemented.

### description

Column names of the query that was run last.

### lastrowid

Returns the row ID of the last inserted row.

### rowcount

Returns the number of rows changed by `INSERT`, `UPDATE`, `DELETE`, and `REPLACE` statements. For other types of statements, returns -1.

### row_factory

Unimplemented.

