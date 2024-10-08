"""libSQL's experimental Python implementation"""

from typing import Any, Self, final


paramstyle = "qmark"
sqlite_version_info = (3, 42, 0)
Error = Exception
LEGACY_TRANSACTION_CONTROL: int = -1


@final
class Cursor:
    """libSQL database cursor.

    Implements a superset of the [DB-API 2.0 (PEP249)](https://peps.python.org/pep-0249/) Cursor object protocol."""  # noqa: E501
    arraysize: int

    @property
    def description(self) -> tuple[tuple[Any, ...], ...] | None: ...

    @property
    def rowcount(self) -> int: ...

    @property
    def lastrowid(self) -> int | None: ...

    def close(self) -> None: ...
    def execute(self, sql: str, parameters: tuple[Any, ...] = ...) -> Self: ...
    def executemany(self, sql: str, parameters: list[tuple[Any, ...]] = ...) -> Self: ...  # noqa: E501
    def executescript(self, script: str) -> Self: ...
    def fetchone(self) -> tuple[Any, ...] | None: ...
    def fetchmany(self, size: int = ...) -> list[tuple[Any, ...]]: ...  # noqa: E501
    def fetchall(self) -> list[tuple[Any, ...]]: ...


@final
class Connection:
    """libSQL database connection.

    Implements a superset of the [DB-API 2.0 (PEP249)](https://peps.python.org/pep-0249/) Connection object protocol."""  # noqa: E501
    @property
    def isolation_level(self) -> str | None: ...

    @property
    def in_transaction(self) -> bool: ...

    def commit(self) -> None: ...
    def cursor(self) -> Cursor: ...
    def sync(self) -> None: ...
    def rollback(self) -> None: ...
    def execute(self, sql: str, parameters: tuple[Any, ...] = ...) -> Cursor: ...  # noqa: E501
    def executemany(self, sql: str, parameters: list[tuple[Any, ...]] = ...) -> Cursor: ...  # noqa: E501
    def executescript(self, script: str) -> None: ...


def connect(database: str,
            isolation_level: str | None = ...,
            check_same_thread: bool = True,
            uri: bool = False,
            sync_url: str = ...,
            sync_interval: float = ...,
            auth_token: str = ...,
            encryption_key: str = ...) -> Connection:
    """Open a new libSQL connection, return a Connection object."""
