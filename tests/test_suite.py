#!/usr/bin/env python3

import sqlite3
import sys
import libsql_experimental
import pytest


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_execute(provider):
    conn = connect(provider, ":memory:")
    conn.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    conn.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = conn.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_cursor_execute(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_executemany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [(1, "alice@example.com"), (2, "bob@example.com")]
    conn.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert [(1, "alice@example.com"), (2, "bob@example.com")] == res.fetchall()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_cursor_fetchone(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [(1, "alice@example.com"), (2, "bob@example.com")]
    cur.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()


@pytest.mark.parametrize("provider", ["sqlite", "libsql"])
def test_cursor_fetchmany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [
        (1, "alice@example.com"),
        (2, "bob@example.com"),
        (3, "carol@example.com"),
        (4, "dave@example.com"),
        (5, "erin@example.com"),
    ]
    cur.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert [(1, "alice@example.com"), (2, "bob@example.com")] == res.fetchmany(2)
    assert [(3, "carol@example.com"), (4, "dave@example.com")] == res.fetchmany(2)
    assert [(5, "erin@example.com")] == res.fetchmany(2)
    assert [] == res.fetchmany(2)


@pytest.mark.parametrize("provider", ["sqlite", "libsql"])
def test_cursor_execute_blob(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, data BLOB)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, b"foobar"))
    res = cur.execute("SELECT * FROM users")
    assert (1, b"foobar") == res.fetchone()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_cursor_executemany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    data = [(1, "alice@example.com"), (2, "bob@example.com")]
    cur.executemany("INSERT INTO users VALUES (?, ?)", data)
    res = cur.execute("SELECT * FROM users")
    assert [(1, "alice@example.com"), (2, "bob@example.com")] == res.fetchall()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_cursor_executescript(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.executescript(
        """
    CREATE TABLE users (id INTEGER, email TEXT);
    INSERT INTO users VALUES (1, 'alice@example.org');
    INSERT INTO users VALUES (2, 'bob@example.org');
    """
    )
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.org") == res.fetchone()
    assert (2, "bob@example.org") == res.fetchone()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_lastrowid(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    assert cur.lastrowid is None
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    assert cur.lastrowid == 0
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    assert cur.lastrowid == 1
    cur.execute("INSERT INTO users VALUES (?, ?)", (2, "bob@example.com"))
    assert cur.lastrowid == 2


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_basic(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT * FROM users")
    assert (
        ("id", None, None, None, None, None, None),
        ("email", None, None, None, None, None, None),
    ) == res.description


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_commit_and_rollback(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    conn.commit()
    cur.execute("INSERT INTO users VALUES (1, 'alice@example.com')")
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()
    conn.rollback()
    res = cur.execute("SELECT * FROM users")
    assert res.fetchone() is None


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_autocommit(provider):
    conn = connect(provider, ":memory:", None)
    assert conn.isolation_level == None
    assert conn.in_transaction == False
    cur = conn.cursor()
    assert conn.in_transaction == False
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "alice@example.com"))
    assert conn.in_transaction == False
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()
    conn.rollback()
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
@pytest.mark.skipif(sys.version_info < (3, 12), reason="requires python3.12 or higher")
def test_connection_autocommit(provider):
    # Test LEGACY_TRANSACTION_CONTROL (-1)
    conn = connect(provider, ":memory:", None, autocommit=-1)
    assert conn.isolation_level is None
    assert conn.autocommit == -1
    cur = conn.cursor()
    assert conn.in_transaction is False
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "alice@example.com"))
    assert conn.in_transaction is False
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()

    conn = connect(provider, ":memory:", isolation_level="DEFERRED", autocommit=-1)
    assert conn.isolation_level == "DEFERRED"
    assert conn.autocommit == -1
    cur = conn.cursor()
    assert conn.in_transaction is False
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "alice@example.com"))
    assert conn.in_transaction is True
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()

    # Test autocommit Enabled (True)
    conn = connect(provider, ":memory:", None, autocommit=True)
    assert conn.isolation_level == None
    assert conn.autocommit == True
    cur = conn.cursor()
    assert conn.in_transaction is False
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "bob@example.com"))
    assert conn.in_transaction is False
    res = cur.execute("SELECT * FROM users")
    assert (1, "bob@example.com") == res.fetchone()

    conn = connect(provider, ":memory:", isolation_level="DEFERRED", autocommit=True)
    assert conn.isolation_level == "DEFERRED"
    assert conn.autocommit == True
    cur = conn.cursor()
    assert conn.in_transaction is False
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "bob@example.com"))
    assert conn.in_transaction is False
    res = cur.execute("SELECT * FROM users")
    assert (1, "bob@example.com") == res.fetchone()

    # Test autocommit Disabled (False)
    conn = connect(provider, ":memory:", isolation_level="DEFERRED", autocommit=False)
    assert conn.isolation_level == "DEFERRED"
    assert conn.autocommit == False
    cur = conn.cursor()
    assert conn.in_transaction is True
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "carol@example.com"))
    assert conn.in_transaction is True
    conn.commit()
    assert conn.in_transaction is True
    res = cur.execute("SELECT * FROM users")
    assert (1, "carol@example.com") == res.fetchone()

    # Test invalid autocommit value (should raise an error)
    with pytest.raises(ValueError):
        connect(provider, ":memory:", None, autocommit=999)


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_params(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "alice@example.com"))
    res = cur.execute("SELECT * FROM users")
    assert (1, "alice@example.com") == res.fetchone()

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_none_param(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, None))
    cur.execute("INSERT INTO users VALUES (?, ?)", (2, "alice@example.com"))
    res = cur.execute("SELECT * FROM users ORDER BY id")
    results = res.fetchall()
    assert results[0] == (1, None)
    assert results[1] == (2, "alice@example.com")

@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_fetchmany(provider):
    conn = connect(provider, ":memory:")
    cur = conn.cursor()
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "alice@example.com"))
    cur.execute("INSERT INTO users VALUES (?, ?)", (2, "bob@example.com"))
    res = cur.execute("SELECT * FROM users")
    assert [(1, "alice@example.com"), (2, "bob@example.com")] == res.fetchall()


@pytest.mark.parametrize("provider", ["libsql", "sqlite"])
def test_in_transaction(provider):
    conn = connect(provider, ":memory:")
    assert conn.in_transaction == False
    cur = conn.cursor()
    assert conn.in_transaction == False
    cur.execute("CREATE TABLE users (id INTEGER, email TEXT)")
    cur.execute("INSERT INTO users VALUES (?, ?)", (1, "alice@example.com"))
    cur.execute("INSERT INTO users VALUES (?, ?)", (2, "bob@example.com"))
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
    cur.execute("INSERT INTO data VALUES (1, 1099511627776)")  # 1 << 40
    res = cur.execute("SELECT * FROM data")
    assert [(1, 1099511627776)] == res.fetchall()

def test_type_adherence(capsys):
    try:
        from mypy.stubtest import test_stubs, parse_options
    except (ImportError, ModuleNotFoundError):
        # skip test if mypy not installed
        pytest.skip("Cant test type stubs without mypy installed")

    # run mypy stubtest tool. Equivalent to running the following the terminal
    """
    stubtest --concise libsql_experimental | \
        grep -v 'which is incompatible with stub argument type'
    """
    test_stubs(parse_options(["--concise", "libsql_experimental"]))
    cap = capsys.readouterr()

    # this is part of error reported if is default parameter is ellipsis
    # `arg: type = ...` which is a nicer way to hide implementation from user
    # than having the more "correct" `arg: type | None = None` everywhere
    ellipsis_err = "which is incompatible with stub argument type"

    lines = cap.out.split("\n")
    lines = filter(lambda x: ellipsis_err not in x, lines)  # filter false positives from ellipsis
    lines = filter(lambda x: len(x) != 0, lines)  # filter empty lines

    # there will always be one error which i dont know how to get rid of
    # `libsql_experimental.libsql_experimental failed to find stubs`
    assert len(list(lines)) == 1

def connect(provider, database, isolation_level="DEFERRED", autocommit=-1):
    if provider == "libsql-remote":
        from urllib import request

        try:
            res = request.urlopen("http://localhost:8080/v2")
        except Exception as _:
            raise Exception("libsql-remote server is not running")
        if res.getcode() != 200:
            raise Exception("libsql-remote server is not running")
        return libsql_experimental.connect(
            database, sync_url="http://localhost:8080", auth_token=""
        )
    if provider == "libsql":
        if sys.version_info < (3, 12):
            return libsql_experimental.connect(
                database, isolation_level=isolation_level
            )
        else:
            if autocommit == -1:
                autocommit = libsql_experimental.LEGACY_TRANSACTION_CONTROL
            return libsql_experimental.connect(
                database, isolation_level=isolation_level, autocommit=autocommit
            )
    if provider == "sqlite":
        if sys.version_info < (3, 12):
            return sqlite3.connect(database, isolation_level=isolation_level)
        else:
            if autocommit == -1:
                autocommit = sqlite3.LEGACY_TRANSACTION_CONTROL
            return sqlite3.connect(
                database, isolation_level=isolation_level, autocommit=autocommit
            )
    raise Exception(f"Provider `{provider}` is not supported")
