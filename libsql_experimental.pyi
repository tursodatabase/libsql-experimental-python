"""Type stubs for libSQL's rust extensions"""

from typing import Any, Sequence, Optional, Literal
from typing_extensions import Self


IsolationLevel = Literal['DEFERRED', 'IMMEDIATE', 'EXCLUSIVE']


class Cursor:
    """libSQL database cursor.

    Implements a superset of the [DB-API 2.0 (PEP249)](https://peps.python.org/pep-0249/) Cursor object protocol."""  # noqa: E501
    @property
    def description(self) -> Optional[Sequence[Sequence[Any]]]: ...

    @property
    def rowcount(self) -> int: ...

    @property
    def arraysize(self) -> int: ...

    @property
    def lastrowid(self) -> Optional[int]: ...

    def close(self) -> None: ...
    def execute(self, sql: str, parameters: Sequence[Any] = ...) -> Self: ...
    def executemany(self, sql: str, parameters: Sequence[Sequence[Any]] = ...) -> Self: ...  # noqa: E501
    def fetchone(self) -> Optional[Sequence[Any]]: ...
    def fetchmany(self, size: int = ...) -> Optional[Sequence[Sequence[Any]]]: ...  # noqa: E501
    def fetchall(self) -> Optional[Sequence[Sequence[Any]]]: ...


class Connection:
    """libSQL database connection.

    Implements a superset of the [DB-API 2.0 (PEP249)](https://peps.python.org/pep-0249/) Connection object protocol."""  # noqa: E501
    @property
    def autocommit(self) -> bool: ...

    @property
    def isolation_level(self) -> Optional[IsolationLevel]: ...

    @property
    def in_transaction(self) -> bool: ...

    def commit(self) -> None: ...
    def cursor(self) -> Cursor: ...
    def sync(self) -> None: ...
    def rollback(self) -> None: ...
    def execute(self, sql: str, parameters: Sequence[Any] = ...) -> Cursor: ...
    def executemany(self, sql: str, parameters: Sequence[Sequence[Any]] = ...) -> Cursor: ...  # noqa: E501
    def executescript(self, script: str) -> None: ...


def connect(database: str,
            isolation_level: Optional[IsolationLevel] = ...,
            check_same_thread: bool = ...,
            uri: bool = ...,
            sync_url: str = ...,
            sync_interval: float = ...,
            auth_token: str = ...,
            encryption_key: str = ...) -> Connection:
    """Open a new libSQL connection, return a Connection object."""
    ...
