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

### execute()

Unimplemented.

### executemany()

Unimplemented.

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

Unimplemented.

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

Execute a SQL statement multiple times for every item in `parameters` array.

### executescript()

Unimplemented.

### fetchone()

Return next row in result set.

### fetchmany()

Unimplemented.

### fetchall()

Return all rows in result set.

### close()

Unimplemented.

### setinputsizes()

Unimplemented.

### setoutputsize()

Unimplemented.

### arraysize

Unimplemented.

### connection

Unimplemented.

### description

Unimplemented.

### lastrowid

Unimplemented.

### rowcount

Unimplemented.

### row_factory

Unimplemented.

