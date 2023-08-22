#!/usr/bin/env python3

import sqlite3
import libsql_experimental
import pytest

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_execute(provider):
    conn = connect(provider, ":memory:")
    conn.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    conn.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = conn.execute("SELECT * FROM users")
    assert (1, 'alice@example.com') == res.fetchone()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_cursor_execute(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT * FROM users")
    assert (1, 'alice@example.com') == res.fetchone()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_basic(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT * FROM users")
    assert (('id', None, None, None, None, None, None), ('email', None, None, None, None, None, None)) == res.description

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_commit_and_rollback(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    conn.commit()
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT * FROM users")
    assert (1, 'alice@example.com') == res.fetchone()
    conn.rollback();
    res = cur.execute("SELECT * FROM users")
    assert res.fetchone() is None

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

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_in_transaction(provider):
    conn = connect(provider, ":memory:")
    assert conn.in_transaction == False
    cur = conn.cursor()
    assert conn.in_transaction == False
    cur.execute('CREATE TABLE users (id INTEGER, email TEXT)')
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, 'alice@example.com'))
    cur.execute("INSERT INTO users VALUES (?, ?)", (2, 'bob@example.com'))
    assert conn.in_transaction == True

def connect(provider, database):
    if provider == "libsql":
        return libsql_experimental.connect(database)
    if provider == "sqlite":
        return sqlite3.connect(database)
    raise Exception(f"Provider `{provider}` is not supported")
