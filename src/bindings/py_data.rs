use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use serde_json::Value as JsonValue;

use crate::glif::data::GlifData;

#[pyclass]
pub struct PyGlifData {
    pub inner: GlifData,
}

#[pymethods]
impl PyGlifData {
    pub fn to_pydict(&self, py: Python) -> PyResult<PyObject> {
        // serialize self.inner to JSON
        let json_val = serde_json::to_value(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        // convert JSON -> PyObject recursively
        json_to_pydict(py, &json_val)
    }
}

fn json_to_pydict<'py>(py: Python<'py>, val: &JsonValue) -> PyResult<PyObject> {
    match val {
        JsonValue::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_to_pydict(py, v)?)?;
            }
            Ok(dict.into())
        }
        JsonValue::Array(arr) => {
            let list = PyList::empty(py);
            for v in arr {
                list.append(json_to_pydict(py, v)?)?;
            }
            Ok(list.into())
        }
        JsonValue::String(s) => Ok(PyString::new(py, s).into()),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(PyInt::new(py, i).into())
            } else if let Some(f) = n.as_f64() {
                Ok(PyFloat::new(py, f).into())
            } else {
                Ok(py.None())
            }
        }
        // PyBool needs to be wrapped in `Py::from`
        JsonValue::Bool(b) => Ok(Py::from(PyBool::new(py, *b)).into()),
        JsonValue::Null => Ok(py.None()),
    }
}
