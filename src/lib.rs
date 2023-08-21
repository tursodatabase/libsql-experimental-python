use ::libsql as libsql_core;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::sync::Arc;

fn to_py_err(error: libsql_core::errors::Error) -> PyErr {
    let msg = match error {
        libsql::Error::PrepareFailed(_, _, err) => err,
        _ => error.to_string(),
    };
    PyValueError::new_err(msg)
}

#[pyfunction]
#[pyo3(signature = (database, check_same_thread=true, uri=false, sync_url=None, sync_auth=""))]
fn connect(
    database: String,
    check_same_thread: bool,
    uri: bool,
    sync_url: Option<String>,
    sync_auth: &str,
) -> PyResult<Connection> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = match sync_url {
        Some(sync_url) => {
            let opts = libsql_core::Opts::with_http_sync(sync_url, sync_auth);
            rt.block_on(libsql_core::Database::open_with_opts(database, opts))
                .map_err(to_py_err)?
        }
        None => libsql_core::Database::open(database).map_err(to_py_err)?,
    };
    let conn = Arc::new(db.connect().map_err(to_py_err)?);
    // TODO: Switch to libSQL transaction API
    conn.execute("BEGIN", ()).map_err(to_py_err)?;
    Ok(Connection { db, conn, rt })
}

#[pyclass]
pub struct Connection {
    db: libsql_core::Database,
    conn: Arc<libsql_core::Connection>,
    rt: tokio::runtime::Runtime,
}

// SAFETY: The libsql crate guarantees that `Connection` is thread-safe.
unsafe impl Send for Connection {}

#[pymethods]
impl Connection {
    fn cursor(self_: PyRef<'_, Self>) -> PyResult<Cursor> {
        Ok(Cursor {
            conn: self_.conn.clone(),
            stmt: None,
            rows: None,
        })
    }

    fn sync(self_: PyRef<'_, Self>) -> PyResult<()> {
        self_.rt.block_on(self_.db.sync()).map_err(to_py_err)?;
        Ok(())
    }

    fn commit(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        self_.conn.execute("COMMIT", ()).map_err(to_py_err)?;
        self_.conn.execute("BEGIN", ()).map_err(to_py_err)?;
        Ok(())
    }

    fn rollback(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO: Switch to libSQL transaction API
        self_.conn.execute("ROLLBACK", ()).map_err(to_py_err)?;
        self_.conn.execute("BEGIN", ()).map_err(to_py_err)?;
        Ok(())
    }
}

#[pyclass]
pub struct Cursor {
    conn: Arc<libsql_core::Connection>,
    stmt: Option<libsql_core::Statement>,
    rows: Option<libsql_core::Rows>,
}

// SAFETY: The libsql crate guarantees that `Connection` is thread-safe.
unsafe impl Send for Cursor {}

#[pymethods]
impl Cursor {
    fn execute<'a>(
        mut self_: PyRefMut<'a, Self>,
        sql: String,
        parameters: Option<&PyTuple>,
    ) -> PyResult<pyo3::PyRefMut<'a, Cursor>> {
        let params: libsql_core::Params = match parameters {
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
                libsql_core::Params::Positional(params)
            }
            None => libsql_core::Params::None,
        };
        let stmt = self_.conn.prepare(sql).map_err(to_py_err)?;
        let rows = stmt.query(&params).map_err(to_py_err)?;
        self_.stmt = Some(stmt);
        self_.rows = Some(rows);
        Ok(self_)
    }

    #[getter]
    fn description(self_: PyRef<'_, Self>) -> PyResult<Option<&PyTuple>> {
        let stmt = self_.stmt.as_ref().unwrap();
        let mut elements: Vec<Py<PyAny>> = vec![];
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

    fn fetchone(self_: PyRef<'_, Self>) -> PyResult<Option<&PyTuple>> {
        match self_.rows {
            Some(ref rows) => {
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

    fn fetchall(self_: PyRef<'_, Self>) -> PyResult<Option<&PyList>> {
        match self_.rows {
            Some(ref rows) => {
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

    fn close(self_: PyRef<'_, Self>) -> PyResult<()> {
        // TODO
        Ok(())
    }
}

fn convert_row(py: Python, row: libsql_core::rows::Row, column_count: i32) -> PyResult<&PyTuple> {
    let mut elements: Vec<Py<PyAny>> = vec![];
    for col_idx in 0..column_count {
        let col_type = row.column_type(col_idx).map_err(to_py_err)?;
        let value = match col_type {
            libsql_core::ValueType::Integer => {
                let value = row.get::<i32>(col_idx).map_err(to_py_err)?;
                value.into_py(py)
            }
            libsql_core::ValueType::Real => todo!(),
            libsql_core::ValueType::Blob => todo!(),
            libsql_core::ValueType::Text => {
                let value = row.get::<&str>(col_idx).map_err(to_py_err)?;
                value.into_py(py)
            }
            libsql_core::ValueType::Null => todo!(),
        };
        elements.push(value);
    }
    Ok(PyTuple::new(py, elements))
}

create_exception!(libsql_experimental, Error, pyo3::exceptions::PyException);

#[pymodule]
fn libsql_experimental(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("paramstyle", "qmark")?;
    m.add("sqlite_version_info", (3, 42, 0))?;
    m.add("Error", py.get_type::<Error>())?;
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_class::<Connection>()?;
    m.add_class::<Cursor>()?;
    Ok(())
}
