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

#[pyfunction]
#[pyo3(signature = (database, isolation_level="DEFERRED", check_same_thread=true, uri=false, sync_url=None, auth_token=""))]
fn connect(
    database: String,
    isolation_level: Option<&str>,
    check_same_thread: bool,
    uri: bool,
    sync_url: Option<String>,
    auth_token: &str,
) -> PyResult<Connection> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = match sync_url {
        Some(sync_url) => {
            let ver = env!("CARGO_PKG_VERSION");
            let ver = format!("libsql-python-rpc-{ver}");

            let fut = libsql::Database::open_with_remote_sync_internal(
                database,
                sync_url,
                auth_token,
                Some(ver),
            );
            let result = rt.block_on(fut);
            result.map_err(to_py_err)?
        }
        None => libsql_core::Database::open(database).map_err(to_py_err)?,
    };
    let autocommit = isolation_level.is_none();
    let conn = db.connect().map_err(to_py_err)?;
    let conn = Arc::new(conn);
    Ok(Connection {
        db,
        conn,
        rt,
        autocommit,
    })
}

#[pyclass]
pub struct Connection {
    db: libsql_core::Database,
    conn: Arc<libsql_core::Connection>,
    rt: tokio::runtime::Runtime,
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

    fn sync(self_: PyRef<'_, Self>) -> PyResult<()> {
        self_.rt.block_on(self_.db.sync()).map_err(to_py_err)?;
        Ok(())
    }

    fn commit(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        if !self_.conn.is_autocommit() {
            self_
                .rt
                .block_on(self_.conn.execute("COMMIT", ()))
                .map_err(to_py_err)?;
        }
        Ok(())
    }

    fn rollback(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        if !self_.conn.is_autocommit() {
            self_
                .rt
                .block_on(self_.conn.execute("ROLLBACK", ()))
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
        rt.block_on(execute(&cursor, sql, parameters))?;
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
                .block_on(execute(&cursor, sql.clone(), Some(parameters)))?;
        }
        Ok(cursor)
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
    conn: Arc<libsql_core::Connection>,
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
        self_.rt.block_on(execute(&self_, sql, parameters))?;
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
                .block_on(execute(&self_, sql.clone(), Some(parameters)))?;
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
                let row = rows.next().map_err(to_py_err)?;
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
                        let row = rows.next().map_err(to_py_err)?;
                        match row {
                            Some(row) => {
                                let row = convert_row(self_.py(), row, rows.column_count())?;
                                elements.push(row.into());
                            }
                            None => {
                                self_.done.replace(true);
                                break
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
                    let row = rows.next().map_err(to_py_err)?;
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
