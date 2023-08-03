#!/usr/bin/env python3

import sqlite3
import libsql_experimental
import pytest

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_basic(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT * FROM users")
    assert (1, 'alice@example.com') == res.fetchone()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_params(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, 'alice@example.com'))
    res = cur.execute("SELECT * FROM users")
    assert (1, 'alice@example.com') == res.fetchone()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_fetchmany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, 'alice@example.com'))
    cur.execute("INSERT INTO users VALUES (?, ?)", (2, 'bob@example.com'))
    res = cur.execute("SELECT * FROM users")
    assert [(1, 'alice@example.com'), (2, 'bob@example.com')] == res.fetchall()

def connect(provider, database):
    if provider == "libsql":
        return libsql_experimental.connect(database)
    if provider == "sqlite":
        return sqlite3.connect(database)
    raise Exception(f"Provider `{provider}` is not supported")
