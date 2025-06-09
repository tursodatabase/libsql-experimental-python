import os
import urllib.parse

from sqlalchemy import util
from sqlalchemy.dialects import registry as _registry
from sqlalchemy.dialects.sqlite.pysqlite import SQLiteDialect_pysqlite

__version__ = "0.1.0-pre"

_registry.register(
    "sqlite.libsql", "sqlalchemy_libsql", "SQLiteDialect_libsql"
)


def _build_connection_url(url, query, secure):
    # sorting of keys is for unit test support
    query_str = urllib.parse.urlencode(sorted(query.items()))

    if not url.host:
        if query_str:
            return f"{url.database}?{query_str}"
        return url.database
    elif secure:  # yes, pop to remove
        scheme = "wss"
    else:
        scheme = "ws"

    if url.username and url.password:
        netloc = f"{url.username}:{url.password}@{url.host}"
    elif url.username:
        netloc = f"{url.username}@{url.host}"
    else:
        netloc = url.host

    if url.port:
        netloc += f":{url.port}"

    return urllib.parse.urlunsplit(
        (
            scheme,
            netloc,
            url.database or "",
            query_str,
            "",  # fragment
        )
    )


class SQLiteDialect_libsql(SQLiteDialect_pysqlite):
    driver = "libsql"
    # need to be set explicitly
    supports_statement_cache = SQLiteDialect_pysqlite.supports_statement_cache

    @classmethod
    def import_dbapi(cls):
        import libsql

        return libsql

    def on_connect(self):
        import libsql

        sqlite3_connect = super().on_connect()

        def connect(conn):
            # LibSQL: there is no support for create_function()
            if isinstance(conn, Connection):
                return
            return sqlite3_connect(conn)

        return connect

    def create_connect_args(self, url):
        pysqlite_args = (
            ("uri", bool),
            ("timeout", float),
            ("isolation_level", str),
            ("detect_types", int),
            ("check_same_thread", bool),
            ("cached_statements", int),
            ("secure", bool),  # LibSQL extra, selects between ws and wss
        )
        opts = url.query
        libsql_opts = {}
        for key, type_ in pysqlite_args:
            util.coerce_kw_type(opts, key, type_, dest=libsql_opts)

        if url.host:
            libsql_opts["uri"] = True

        if libsql_opts.get("uri", False):
            uri_opts = dict(opts)
            # here, we are actually separating the parameters that go to
            # sqlite3/pysqlite vs. those that go the SQLite URI.  What if
            # two names conflict?  again, this seems to be not the case right
            # now, and in the case that new names are added to
            # either side which overlap, again the sqlite3/pysqlite parameters
            # can be passed through connect_args instead of in the URL.
            # If SQLite native URIs add a parameter like "timeout" that
            # we already have listed here for the python driver, then we need
            # to adjust for that here.
            for key, type_ in pysqlite_args:
                uri_opts.pop(key, None)

            secure = libsql_opts.pop("secure", False)
            connect_url = _build_connection_url(url, uri_opts, secure)
        else:
            connect_url = url.database or ":memory:"
            if connect_url != ":memory:":
                connect_url = os.path.abspath(connect_url)

        libsql_opts.setdefault(
            "check_same_thread", not self._is_url_file_db(url)
        )

        return ([connect_url], libsql_opts)


dialect = SQLiteDialect_libsql
