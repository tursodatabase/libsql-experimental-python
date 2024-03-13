use ::libsql as libsql_core;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::cell::RefCell;
use std::sync::Arc;

fn to_py_err(error: libsql_core::errors::Error) -> PyErr {
    let msg = match error {
        libsql::Error::SqliteFailure(_, err) => err,
        _ => error.to_string(),
    };
    PyValueError::new_err(msg)
}

fn is_remote_path(path: &str) -> bool {
    path.starts_with("libsql://") || path.starts_with("http://") || path.starts_with("https://")
}

#[pyfunction]
#[pyo3(signature = (database, isolation_level="DEFERRED".to_string(), check_same_thread=true, uri=false, sync_url=None, auth_token=""))]
fn connect(
    py: Python<'_>,
    database: String,
    isolation_level: Option<String>,
    check_same_thread: bool,
    uri: bool,
    sync_url: Option<String>,
    auth_token: &str,
) -> PyResult<Connection> {
    let ver = env!("CARGO_PKG_VERSION");
    let ver = format!("libsql-python-rpc-{ver}");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = if is_remote_path(&database) {
        let result = libsql::Database::open_remote_internal(database.clone(), auth_token, ver);
        result.map_err(to_py_err)?
    } else {
        match sync_url {
            Some(sync_url) => {
                let fut = libsql::Database::open_with_remote_sync_internal(
                    database,
                    sync_url,
                    auth_token,
                    Some(ver),
                    true,
                    None,
                    None,
                );
                tokio::pin!(fut);
                let result = rt.block_on(check_signals(py, fut));
                result.map_err(to_py_err)?
            }
            None => libsql_core::Database::open(database).map_err(to_py_err)?,
        }
    };
    let autocommit = isolation_level.is_none();
    let conn = db.connect().map_err(to_py_err)?;
    Ok(Connection {
        db,
        conn: Arc::new(ConnectionGuard {
            conn: Some(conn),
            handle: rt.handle().clone(),
        }),
        rt,
        isolation_level,
        autocommit,
    })
}

// We need to add a drop guard that runs when we finally drop our
// only reference to libsql_core::Connection. This is because when
// hrana is enabled it needs access to the tokio api to spawn a close
// call in the background. So this adds the ability that when drop is called
// on ConnectionGuard it will drop the connection with a tokio context entered.
struct ConnectionGuard {
    conn: Option<libsql_core::Connection>,
    handle: tokio::runtime::Handle,
}

impl std::ops::Deref for ConnectionGuard {
    type Target = libsql_core::Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn.as_ref().expect("Connection already dropped")
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let _enter = self.handle.enter();
        if let Some(conn) = self.conn.take() {
            drop(conn);
        }
    }
}

#[pyclass]
pub struct Connection {
    db: libsql_core::Database,
    conn: Arc<ConnectionGuard>,
    rt: tokio::runtime::Runtime,
    isolation_level: Option<String>,
    autocommit: bool,
}

// SAFETY: The libsql crate guarantees that `Connection` is thread-safe.
unsafe impl Send for Connection {}

#[pymethods]
impl Connection {
    fn cursor(&self) -> PyResult<Cursor> {
        Ok(Cursor {
            arraysize: 1,
            rt: self.rt.handle().clone(),
            conn: self.conn.clone(),
            stmt: RefCell::new(None),
            rows: RefCell::new(None),
            rowcount: RefCell::new(0),
            autocommit: self.autocommit,
            done: RefCell::new(false),
        })
    }

    fn sync(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<()> {
        let fut = {
            let _enter = self_.rt.enter();
            self_.db.sync()
        };
        tokio::pin!(fut);

        self_
            .rt
            .block_on(check_signals(py, fut))
            .map_err(to_py_err)?;
        Ok(())
    }

    fn commit(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        if !self_.conn.is_autocommit() {
            self_
                .rt
                .block_on(async { self_.conn.execute("COMMIT", ()).await })
                .map_err(to_py_err)?;
        }
        Ok(())
    }

    fn rollback(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        if !self_.conn.is_autocommit() {
            self_
                .rt
                .block_on(async { self_.conn.execute("ROLLBACK", ()).await })
                .map_err(to_py_err)?;
        }
        Ok(())
    }

    fn execute(
        self_: PyRef<'_, Self>,
        sql: String,
        parameters: Option<&PyTuple>,
    ) -> PyResult<Cursor> {
        let cursor = Connection::cursor(&self_)?;
        let rt = self_.rt.handle();
        rt.block_on(async { execute(&cursor, sql, parameters).await })?;
        Ok(cursor)
    }

    fn executemany(
        self_: PyRef<'_, Self>,
        sql: String,
        parameters: Option<&PyList>,
    ) -> PyResult<Cursor> {
        let cursor = Connection::cursor(&self_)?;
        for parameters in parameters.unwrap().iter() {
            let parameters = parameters.extract::<&PyTuple>()?;
            self_
                .rt
                .block_on(async { execute(&cursor, sql.clone(), Some(parameters)).await })?;
        }
        Ok(cursor)
    }

    fn executescript(self_: PyRef<'_, Self>, script: String) -> PyResult<()> {
        let statements = script.split(';');
        for statement in statements {
            let statement = statement.trim();
            if !statement.is_empty() {
                let cursor = Connection::cursor(&self_)?;
                self_
                    .rt
                    .block_on(async { execute(&cursor, statement.to_string(), None).await })?;
            }
        }
        Ok(())
    }

    #[getter]
    fn isolation_level(self_: PyRef<'_, Self>) -> Option<String> {
        self_.isolation_level.clone()
    }

    #[getter]
    fn in_transaction(self_: PyRef<'_, Self>) -> PyResult<bool> {
        Ok(!self_.conn.is_autocommit())
    }
}

#[pyclass]
pub struct Cursor {
    #[pyo3(get, set)]
    arraysize: usize,
    rt: tokio::runtime::Handle,
    conn: Arc<ConnectionGuard>,
    stmt: RefCell<Option<libsql_core::Statement>>,
    rows: RefCell<Option<libsql_core::Rows>>,
    rowcount: RefCell<i64>,
    done: RefCell<bool>,
    autocommit: bool,
}

// SAFETY: The libsql crate guarantees that `Connection` is thread-safe.
unsafe impl Send for Cursor {}

#[pymethods]
impl Cursor {
    fn execute<'a>(
        self_: PyRef<'a, Self>,
        sql: String,
        parameters: Option<&PyTuple>,
    ) -> PyResult<pyo3::PyRef<'a, Self>> {
        self_
            .rt
            .block_on(async { execute(&self_, sql, parameters).await })?;
        Ok(self_)
    }

    fn executemany<'a>(
        self_: PyRef<'a, Self>,
        sql: String,
        parameters: Option<&PyList>,
    ) -> PyResult<pyo3::PyRef<'a, Cursor>> {
        for parameters in parameters.unwrap().iter() {
            let parameters = parameters.extract::<&PyTuple>()?;
            self_
                .rt
                .block_on(async { execute(&self_, sql.clone(), Some(parameters)).await })?;
        }
        Ok(self_)
    }

    #[getter]
    fn description(self_: PyRef<'_, Self>) -> PyResult<Option<&PyTuple>> {
        let stmt = self_.stmt.borrow();
        let mut elements: Vec<Py<PyAny>> = vec![];
        for column in stmt.as_ref().unwrap().columns() {
            let name = column.name();
            let element = (
                name,
                self_.py().None(),
                self_.py().None(),
                self_.py().None(),
                self_.py().None(),
                self_.py().None(),
                self_.py().None(),
            )
                .to_object(self_.py());
            elements.push(element);
        }
        let elements = PyTuple::new(self_.py(), elements);
        Ok(Some(elements))
    }

    fn fetchone(self_: PyRef<'_, Self>) -> PyResult<Option<&PyTuple>> {
        let mut rows = self_.rows.borrow_mut();
        match rows.as_mut() {
            Some(rows) => {
                let row = self_.rt.block_on(rows.next()).map_err(to_py_err)?;
                match row {
                    Some(row) => {
                        let row = convert_row(self_.py(), row, rows.column_count())?;
                        Ok(Some(row))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    fn fetchmany(self_: PyRef<'_, Self>, size: Option<i64>) -> PyResult<Option<&PyList>> {
        let mut rows = self_.rows.borrow_mut();
        match rows.as_mut() {
            Some(rows) => {
                let size = size.unwrap_or(self_.arraysize as i64);
                let mut elements: Vec<Py<PyAny>> = vec![];
                // The libSQL Rows.next() method restarts the iteration if it
                // has reached the end, which is why we need to check if we're
                // done before iterating.
                if !*self_.done.borrow() {
                    for _ in 0..size {
                        let row = self_
                            .rt
                            .block_on(async { rows.next().await })
                            .map_err(to_py_err)?;
                        match row {
                            Some(row) => {
                                let row = convert_row(self_.py(), row, rows.column_count())?;
                                elements.push(row.into());
                            }
                            None => {
                                self_.done.replace(true);
                                break;
                            }
                        }
                    }
                }
                Ok(Some(PyList::new(self_.py(), elements)))
            }
            None => Ok(None),
        }
    }

    fn fetchall(self_: PyRef<'_, Self>) -> PyResult<Option<&PyList>> {
        let mut rows = self_.rows.borrow_mut();
        match rows.as_mut() {
            Some(rows) => {
                let mut elements: Vec<Py<PyAny>> = vec![];
                loop {
                    let row = self_
                        .rt
                        .block_on(async { rows.next().await })
                        .map_err(to_py_err)?;
                    match row {
                        Some(row) => {
                            let row = convert_row(self_.py(), row, rows.column_count())?;
                            elements.push(row.into());
                        }
                        None => break,
                    }
                }
                Ok(Some(PyList::new(self_.py(), elements)))
            }
            None => Ok(None),
        }
    }

    #[getter]
    fn lastrowid(self_: PyRef<'_, Self>) -> PyResult<Option<i64>> {
        let stmt = self_.stmt.borrow();
        match stmt.as_ref() {
            Some(_) => Ok(Some(self_.conn.last_insert_rowid())),
            None => Ok(None),
        }
    }

    #[getter]
    fn rowcount(self_: PyRef<'_, Self>) -> PyResult<i64> {
        Ok(*self_.rowcount.borrow())
    }

    fn close(_self: PyRef<'_, Self>) -> PyResult<()> {
        // TODO
        Ok(())
    }
}

async fn begin_transaction(conn: &libsql_core::Connection) -> PyResult<()> {
    conn.execute("BEGIN", ()).await.map_err(to_py_err)?;
    Ok(())
}

async fn execute(cursor: &Cursor, sql: String, parameters: Option<&PyTuple>) -> PyResult<()> {
    let stmt_is_dml = stmt_is_dml(&sql);
    if !cursor.autocommit && stmt_is_dml && cursor.conn.is_autocommit() {
        begin_transaction(&cursor.conn).await?;
    }
    let params = match parameters {
        Some(parameters) => {
            let mut params = vec![];
            for parameter in parameters.iter() {
                let param = match parameter.extract::<i32>() {
                    Ok(value) => libsql_core::Value::Integer(value as i64),
                    Err(_) => match parameter.extract::<f64>() {
                        Ok(value) => libsql_core::Value::Real(value),
                        Err(_) => match parameter.extract::<&str>() {
                            Ok(value) => libsql_core::Value::Text(value.to_string()),
                            Err(_) => todo!(),
                        },
                    },
                };
                params.push(param);
            }
            libsql_core::params::Params::Positional(params)
        }
        None => libsql_core::params::Params::None,
    };
    let mut stmt = cursor.conn.prepare(&sql).await.map_err(to_py_err)?;
    let rows = stmt.query(params).await.map_err(to_py_err)?;
    if stmt_is_dml {
        let mut rowcount = cursor.rowcount.borrow_mut();
        *rowcount += cursor.conn.changes() as i64;
    } else {
        cursor.rowcount.replace(-1);
    }
    cursor.stmt.replace(Some(stmt));
    cursor.rows.replace(Some(rows));
    Ok(())
}

fn stmt_is_dml(sql: &str) -> bool {
    let sql = sql.trim();
    let sql = sql.to_uppercase();
    sql.starts_with("INSERT") || sql.starts_with("UPDATE") || sql.starts_with("DELETE")
}

fn convert_row(py: Python, row: libsql_core::Row, column_count: i32) -> PyResult<&PyTuple> {
    let mut elements: Vec<Py<PyAny>> = vec![];
    for col_idx in 0..column_count {
        let col_type = row.column_type(col_idx).map_err(to_py_err)?;
        let value = match col_type {
            libsql::ValueType::Integer => {
                let value = row.get::<i32>(col_idx).map_err(to_py_err)?;
                value.into_py(py)
            }
            libsql::ValueType::Real => {
                let value = row.get::<f64>(col_idx).map_err(to_py_err)?;
                value.into_py(py)
            }
            libsql::ValueType::Blob => todo!("blobs not supported"),
            libsql::ValueType::Text => {
                let value = row.get::<String>(col_idx).map_err(to_py_err)?;
                value.into_py(py)
            }
            libsql::ValueType::Null => py.None(),
        };
        elements.push(value);
    }
    Ok(PyTuple::new(py, elements))
}

create_exception!(libsql_experimental, Error, pyo3::exceptions::PyException);

#[pymodule]
fn libsql_experimental(py: Python, m: &PyModule) -> PyResult<()> {
    let _ = tracing_subscriber::fmt::try_init();
    m.add("paramstyle", "qmark")?;
    m.add("sqlite_version_info", (3, 42, 0))?;
    m.add("Error", py.get_type::<Error>())?;
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_class::<Connection>()?;
    m.add_class::<Cursor>()?;
    Ok(())
}

async fn check_signals<F, R>(py: Python<'_>, mut fut: std::pin::Pin<&mut F>) -> R
where
    F: std::future::Future<Output = R>,
{
    loop {
        tokio::select! {
            out = &mut fut => {
                break out;
            }

            _ = tokio::time::sleep(std::time::Duration::from_millis(300)) => {
                py.check_signals().unwrap();
            }
        }
    }
}
