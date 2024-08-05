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
def test_executemany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [
        (1, 'alice@example.com'),
        (2, 'bob@example.com')
    ]
    conn.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert [(1, 'alice@example.com'), (2, 'bob@example.com')] == res.fetchall()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_cursor_fetchone(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [
        (1, 'alice@example.com'),
        (2, 'bob@example.com')
    ]
    cur.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert (1, 'alice@example.com') == res.fetchone()

@pytest.mark.parametrize("provider", ["sqlite", "libsql"])
def test_cursor_fetchmany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [
        (1, 'alice@example.com'),
        (2, 'bob@example.com'),
        (3, 'carol@example.com'),
        (4, 'dave@example.com'),
        (5, 'erin@example.com')
    ]
    cur.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert [(1, 'alice@example.com'), (2, 'bob@example.com')] == res.fetchmany(2)
    assert [(3, 'carol@example.com'), (4, 'dave@example.com')] == res.fetchmany(2)
    assert [(5, 'erin@example.com')] == res.fetchmany(2)
    assert [] == res.fetchmany(2)

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_cursor_executemany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [
        (1, 'alice@example.com'),
        (2, 'bob@example.com')
    ]
    cur.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert [(1, 'alice@example.com'), (2, 'bob@example.com')] == res.fetchall()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_lastrowid(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    assert cur.lastrowid is None
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    assert cur.lastrowid == 0
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    assert cur.lastrowid == 1
    cur.execute("INSERT INTO users VALUES (?, ?)", (2, 'bob@example.com'))
    assert cur.lastrowid == 2

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
def test_autocommit(provider):
    conn = connect(provider, ":memory:", None)
    assert conn.isolation_level == None
    assert conn.in_transaction == False
    cur = conn.cursor()
    assert conn.in_transaction == False
    cur.execute('CREATE TABLE users (id INTEGER, email TEXT)')
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, 'alice@example.com'))
    assert conn.in_transaction == False
    res = cur.execute("SELECT * FROM users")
    assert (1, 'alice@example.com') == res.fetchone()
    conn.rollback();
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

@pytest.mark.parametrize("provider", ["libsql-remote", "libsql", "sqlite"])
def test_fetch_expression(provider):
    dbname = "/tmp/test.db" if provider == "libsql-remote" else ":memory:"
    try:
        conn = connect(provider, dbname)
    except Exception as e:
        pytest.skip(str(e))
    cur = conn.cursor()
    cur.execute("DROP TABLE IF EXISTS users")
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT QUOTE(email) FROM users")
    assert [("'alice@example.com'",)] == res.fetchall()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_int64(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE data (id INTEGER, number INTEGER)")
    conn.commit()
    cur.execute("INSERT INTO data VALUES (1, 1099511627776)") # 1 << 40
    res = cur.execute("SELECT * FROM data")
    assert [(1,1099511627776)] == res.fetchall()

def connect(provider, database, isolation_level='DEFERRED'):
    if provider == "libsql-remote":
        from urllib import request
        try:
            res = request.urlopen("http://localhost:8080/v2")
        except Exception as _:
            raise Exception("libsql-remote server is not running")
        if res.getcode() != 200:
            raise Exception("libsql-remote server is not running")
        return libsql_experimental.connect(database, sync_url="http://localhost:8080", auth_token="")
    if provider == "libsql":
        return libsql_experimental.connect(database, isolation_level = isolation_level)
    if provider == "sqlite":
        return sqlite3.connect(database, isolation_level = isolation_level)
    raise Exception(f"Provider `{provider}` is not supported")
