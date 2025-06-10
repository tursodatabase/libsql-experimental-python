use ::libsql as libsql_core;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::cell::{OnceCell, RefCell};
use std::sync::{Arc, OnceLock};
use tokio::runtime::{Handle, Runtime};

const LEGACY_TRANSACTION_CONTROL: i32 = -1;

fn rt() -> Handle {
    static RT: OnceLock<Runtime> = OnceLock::new();

    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
    })
    .handle()
    .clone()
}

fn to_py_err(error: libsql_core::errors::Error) -> PyErr {
    let msg = match error {
        libsql_core::Error::SqliteFailure(_, err) => err,
        _ => error.to_string(),
    };
    PyValueError::new_err(msg)
}

fn is_remote_path(path: &str) -> bool {
    path.starts_with("libsql://") || path.starts_with("http://") || path.starts_with("https://")
}

#[pyfunction]
#[cfg(not(Py_3_12))]
#[pyo3(signature = (database, isolation_level="DEFERRED".to_string(), check_same_thread=true, uri=false, sync_url=None, sync_interval=None, auth_token="", encryption_key=None))]
fn connect(
    py: Python<'_>,
    database: String,
    isolation_level: Option<String>,
    check_same_thread: bool,
    uri: bool,
    sync_url: Option<String>,
    sync_interval: Option<f64>,
    auth_token: &str,
    encryption_key: Option<String>,
) -> PyResult<Connection> {
    let conn = _connect_core(
        py,
        database,
        isolation_level,
        check_same_thread,
        uri,
        sync_url,
        sync_interval,
        auth_token,
        encryption_key,
    )?;
    Ok(conn)
}

#[pyfunction]
#[cfg(Py_3_12)]
#[pyo3(signature = (database, isolation_level="DEFERRED".to_string(), check_same_thread=true, uri=false, sync_url=None, sync_interval=None, auth_token="", encryption_key=None, autocommit = LEGACY_TRANSACTION_CONTROL))]
fn connect(
    py: Python<'_>,
    database: String,
    isolation_level: Option<String>,
    check_same_thread: bool,
    uri: bool,
    sync_url: Option<String>,
    sync_interval: Option<f64>,
    auth_token: &str,
    encryption_key: Option<String>,
    autocommit: i32,
) -> PyResult<Connection> {
    let mut conn = _connect_core(
        py,
        database,
        isolation_level.clone(),
        check_same_thread,
        uri,
        sync_url,
        sync_interval,
        auth_token,
        encryption_key,
    )?;

    conn.autocommit =
        if autocommit == LEGACY_TRANSACTION_CONTROL || autocommit == 1 || autocommit == 0 {
            autocommit
        } else {
            return Err(PyValueError::new_err(
                "autocommit must be True, False, or sqlite3.LEGACY_TRANSACTION_CONTROL",
            ));
        };
    Ok(conn)
}

fn _connect_core(
    py: Python<'_>,
    database: String,
    isolation_level: Option<String>,
    check_same_thread: bool,
    uri: bool,
    sync_url: Option<String>,
    sync_interval: Option<f64>,
    auth_token: &str,
    encryption_key: Option<String>,
) -> PyResult<Connection> {
    let ver = env!("CARGO_PKG_VERSION");
    let ver = format!("libsql-python-rpc-{ver}");
    let rt = rt();
    let encryption_config = match encryption_key {
        Some(key) => {
            let cipher = libsql_core::Cipher::default();
            let encryption_config = libsql_core::EncryptionConfig::new(cipher, key.into());
            Some(encryption_config)
        }
        None => None,
    };
    let db = if is_remote_path(&database) {
        let result = libsql_core::Database::open_remote_internal(database.clone(), auth_token, ver);
        result.map_err(to_py_err)?
    } else {
        match sync_url {
            Some(sync_url) => {
                let sync_interval = sync_interval.map(|i| std::time::Duration::from_secs_f64(i));
                let mut builder =
                    libsql_core::Builder::new_remote_replica(database, sync_url, auth_token.to_string());
                if let Some(encryption_config) = encryption_config {
                    builder = builder.encryption_config(encryption_config);
                }
                if let Some(sync_interval) = sync_interval {
                    builder = builder.sync_interval(sync_interval);
                }
                let fut = builder.build();
                tokio::pin!(fut);
                let result = rt.block_on(check_signals(py, fut));
                result.map_err(to_py_err)?
            }
            None => {
                let mut builder = libsql_core::Builder::new_local(database);
                if let Some(config) = encryption_config {
                    builder = builder.encryption_config(config);
                }
                let fut = builder.build();
                tokio::pin!(fut);
                let result = rt.block_on(check_signals(py, fut));
                result.map_err(to_py_err)?
            }
        }
    };

    let autocommit = isolation_level.is_none() as i32;
    let conn = db.connect().map_err(to_py_err)?;
    Ok(Connection {
        db,
        conn: RefCell::new(Some(Arc::new(ConnectionGuard {
            conn: Some(conn),
            handle: rt.clone(),
        }))),
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
    conn: RefCell<Option<Arc<ConnectionGuard>>>,
    isolation_level: Option<String>,
    autocommit: i32,
}

// SAFETY: The libsql crate guarantees that `Connection` is thread-safe.
unsafe impl Send for Connection {}

#[pymethods]
impl Connection {
    fn close(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<()> {
        self_.conn.replace(None);
        Ok(())
    }

    fn cursor(&self) -> PyResult<Cursor> {
        Ok(Cursor {
            arraysize: 1,
            conn: RefCell::new(Some(self.conn.borrow().as_ref().unwrap().clone())),
            stmt: RefCell::new(None),
            rows: RefCell::new(None),
            rowcount: RefCell::new(0),
            autocommit: self.autocommit,
            isolation_level: self.isolation_level.clone(),
            done: RefCell::new(false),
        })
    }

    fn sync(self_: PyRef<'_, Self>, py: Python<'_>) -> PyResult<()> {
        let fut = {
            let _enter = rt().enter();
            self_.db.sync()
        };
        tokio::pin!(fut);

        rt().block_on(check_signals(py, fut)).map_err(to_py_err)?;
        Ok(())
    }

    fn commit(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        if !self_.conn.borrow().as_ref().unwrap().is_autocommit() {
            rt().block_on(async {
                self_
                    .conn
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .execute("COMMIT", ())
                    .await
            })
            .map_err(to_py_err)?;
        }
        Ok(())
    }

    fn rollback(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        if !self_.conn.borrow().as_ref().unwrap().is_autocommit() {
            rt().block_on(async {
                self_
                    .conn
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .execute("ROLLBACK", ())
                    .await
            })
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
        rt().block_on(async { execute(&cursor, sql, parameters).await })?;
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
            rt().block_on(async { execute(&cursor, sql.clone(), Some(parameters)).await })?;
        }
        Ok(cursor)
    }

    fn executescript(self_: PyRef<'_, Self>, script: String) -> PyResult<()> {
        let _ = rt()
            .block_on(async {
                self_
                    .conn
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .execute_batch(&script)
                    .await
            })
            .map_err(to_py_err);
        Ok(())
    }

    #[getter]
    fn isolation_level(self_: PyRef<'_, Self>) -> Option<String> {
        self_.isolation_level.clone()
    }

    #[getter]
    fn in_transaction(self_: PyRef<'_, Self>) -> PyResult<bool> {
        #[cfg(Py_3_12)]
        {
            return Ok(
                !self_.conn.borrow().as_ref().unwrap().is_autocommit() || self_.autocommit == 0
            );
        }
        Ok(!self_.conn.borrow().as_ref().unwrap().is_autocommit())
    }

    #[getter]
    #[cfg(Py_3_12)]
    fn get_autocommit(self_: PyRef<'_, Self>) -> PyResult<i32> {
        Ok(self_.autocommit)
    }

    #[setter]
    #[cfg(Py_3_12)]
    fn set_autocommit(mut self_: PyRefMut<'_, Self>, autocommit: i32) -> PyResult<()> {
        if autocommit != LEGACY_TRANSACTION_CONTROL && autocommit != 1 && autocommit != 0 {
            return Err(PyValueError::new_err(
                "autocommit must be True, False, or sqlite3.LEGACY_TRANSACTION_CONTROL",
            ));
        }
        self_.autocommit = autocommit;
        Ok(())
    }
}

#[pyclass]
pub struct Cursor {
    #[pyo3(get, set)]
    arraysize: usize,
    conn: RefCell<Option<Arc<ConnectionGuard>>>,
    stmt: RefCell<Option<libsql_core::Statement>>,
    rows: RefCell<Option<libsql_core::Rows>>,
    rowcount: RefCell<i64>,
    done: RefCell<bool>,
    isolation_level: Option<String>,
    autocommit: i32,
}

// SAFETY: The libsql crate guarantees that `Connection` is thread-safe.
unsafe impl Send for Cursor {}

impl Drop for Cursor {
    fn drop(&mut self) {
        let _enter = rt().enter();
        self.conn.replace(None);
        self.stmt.replace(None);
        self.rows.replace(None);
    }
}

#[pymethods]
impl Cursor {
    fn close(self_: PyRef<'_, Self>) -> PyResult<()> {
        rt().block_on(async {
            let cursor: &Cursor = &self_;
            cursor.conn.replace(None);
            cursor.stmt.replace(None);
            cursor.rows.replace(None);
        });
        Ok(())
    }

    fn execute<'a>(
        self_: PyRef<'a, Self>,
        sql: String,
        parameters: Option<&PyTuple>,
    ) -> PyResult<pyo3::PyRef<'a, Self>> {
        rt().block_on(async { execute(&self_, sql, parameters).await })?;
        Ok(self_)
    }

    fn executemany<'a>(
        self_: PyRef<'a, Self>,
        sql: String,
        parameters: Option<&PyList>,
    ) -> PyResult<pyo3::PyRef<'a, Cursor>> {
        for parameters in parameters.unwrap().iter() {
            let parameters = parameters.extract::<&PyTuple>()?;
            rt().block_on(async { execute(&self_, sql.clone(), Some(parameters)).await })?;
        }
        Ok(self_)
    }

    fn executescript<'a>(
        self_: PyRef<'a, Self>,
        script: String,
    ) -> PyResult<pyo3::PyRef<'a, Self>> {
        rt().block_on(async {
            self_
                .conn
                .borrow()
                .as_ref()
                .unwrap()
                .execute_batch(&script)
                .await
        })
        .map_err(to_py_err)?;
        Ok(self_)
    }

    #[getter]
    fn description(self_: PyRef<'_, Self>) -> PyResult<Option<&PyTuple>> {
        let stmt = self_.stmt.borrow();
        let mut elements: Vec<Py<PyAny>> = vec![];
        match stmt.as_ref() {
            Some(stmt) => {
                for column in stmt.columns() {
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
            None => Ok(None),
        }
    }

    fn fetchone(self_: PyRef<'_, Self>) -> PyResult<Option<&PyTuple>> {
        let mut rows = self_.rows.borrow_mut();
        match rows.as_mut() {
            Some(rows) => {
                let row = rt().block_on(rows.next()).map_err(to_py_err)?;
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
                        let row = rt()
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
                    let row = rt()
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
            Some(_) => Ok(Some(
                self_.conn.borrow().as_ref().unwrap().last_insert_rowid(),
            )),
            None => Ok(None),
        }
    }

    #[getter]
    fn rowcount(self_: PyRef<'_, Self>) -> PyResult<i64> {
        Ok(*self_.rowcount.borrow())
    }
}

async fn begin_transaction(conn: &libsql_core::Connection) -> PyResult<()> {
    conn.execute("BEGIN", ()).await.map_err(to_py_err)?;
    Ok(())
}

async fn execute(cursor: &Cursor, sql: String, parameters: Option<&PyTuple>) -> PyResult<()> {
    if cursor.conn.borrow().as_ref().is_none() {
        return Err(PyValueError::new_err("Connection already closed"));
    }
    let stmt_is_dml = stmt_is_dml(&sql);
    let autocommit = determine_autocommit(cursor);
    if !autocommit && stmt_is_dml && cursor.conn.borrow().as_ref().unwrap().is_autocommit() {
        begin_transaction(&cursor.conn.borrow().as_ref().unwrap()).await?;
    }
    let params = match parameters {
        Some(parameters) => {
            let mut params = vec![];
            for param in parameters.iter() {
                let param = if param.is_none() {
                    libsql_core::Value::Null
                } else if let Ok(value) = param.extract::<i32>() {
                    libsql_core::Value::Integer(value as i64)
                } else if let Ok(value) = param.extract::<f64>() {
                    libsql_core::Value::Real(value)
                } else if let Ok(value) = param.extract::<&str>() {
                    libsql_core::Value::Text(value.to_string())
                } else if let Ok(value) = param.extract::<&[u8]>() {
                    libsql_core::Value::Blob(value.to_vec())
                } else {
                    return Err(PyValueError::new_err("Unsupported parameter type"));
                };
                params.push(param);
            }
            libsql_core::params::Params::Positional(params)
        }
        None => libsql_core::params::Params::None,
    };
    let mut stmt = cursor
        .conn
        .borrow()
        .as_ref()
        .unwrap()
        .prepare(&sql)
        .await
        .map_err(to_py_err)?;

    if stmt.columns().iter().len() > 0 {
        let rows = stmt.query(params).await.map_err(to_py_err)?;
        cursor.rows.replace(Some(rows));
    } else {
        stmt.execute(params).await.map_err(to_py_err)?;
        cursor.rows.replace(None);
    }

    let mut rowcount = cursor.rowcount.borrow_mut();
    *rowcount += cursor.conn.borrow().as_ref().unwrap().changes() as i64;

    cursor.stmt.replace(Some(stmt));
    Ok(())
}

fn determine_autocommit(cursor: &Cursor) -> bool {
    #[cfg(Py_3_12)]
    {
        match cursor.autocommit {
            LEGACY_TRANSACTION_CONTROL => cursor.isolation_level.is_none(),
            _ => cursor.autocommit != 0,
        }
    }

    #[cfg(not(Py_3_12))]
    {
        cursor.isolation_level.is_none()
    }
}

fn stmt_is_dml(sql: &str) -> bool {
    let sql = sql.trim();
    let sql = sql.to_uppercase();
    sql.starts_with("INSERT") || sql.starts_with("UPDATE") || sql.starts_with("DELETE")
}

fn convert_row(py: Python, row: libsql_core::Row, column_count: i32) -> PyResult<&PyTuple> {
    let mut elements: Vec<Py<PyAny>> = vec![];
    for col_idx in 0..column_count {
        let libsql_value = row.get_value(col_idx).map_err(to_py_err)?;
        let value = match libsql_value {
            libsql_core::Value::Integer(v) => {
                let value = v as i64;
                value.into_py(py)
            }
            libsql_core::Value::Real(v) => v.into_py(py),
            libsql_core::Value::Text(v) => v.into_py(py),
            libsql_core::Value::Blob(v) => {
                let value = v.as_slice();
                value.into_py(py)
            }
            libsql_core::Value::Null => py.None(),
        };
        elements.push(value);
    }
    Ok(PyTuple::new(py, elements))
}

create_exception!(libsql, Error, pyo3::exceptions::PyException);

#[pymodule]
fn libsql(py: Python, m: &PyModule) -> PyResult<()> {
    let _ = tracing_subscriber::fmt::try_init();
    m.add("LEGACY_TRANSACTION_CONTROL", LEGACY_TRANSACTION_CONTROL)?;
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
