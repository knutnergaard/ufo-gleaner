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

#[cfg(test)]
mod tests {
    use crate::glif::Advance;

    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList};
    use serde_json::json;

    fn setup_python() -> Python<'static> {
        pyo3::prepare_freethreaded_python();
        unsafe { Python::assume_gil_acquired() }
    }

    #[test]
    fn test_json_to_pydict_basic_types() {
        Python::with_gil(|py| {
            let val = json!({
                "string": "hello",
                "int": 42,
                "float": 3.14,
                "bool": true,
                "null": null
            });

            let obj: Py<PyAny> = json_to_pydict(py, &val).unwrap();
            let dict = obj.downcast_bound::<PyDict>(py).unwrap();

            assert_eq!(
                dict.get_item("string")
                    .unwrap()
                    .unwrap()
                    .extract::<&str>()
                    .unwrap(),
                "hello"
            );
            assert_eq!(
                dict.get_item("int")
                    .unwrap()
                    .unwrap()
                    .extract::<i64>()
                    .unwrap(),
                42
            );
            assert_eq!(
                dict.get_item("float")
                    .unwrap()
                    .unwrap()
                    .extract::<f64>()
                    .unwrap(),
                3.14
            );
            assert_eq!(
                dict.get_item("bool")
                    .unwrap()
                    .unwrap()
                    .extract::<bool>()
                    .unwrap(),
                true
            );
            assert!(dict.get_item("null").unwrap().is_none());
        });
    }

    #[test]
    fn test_json_to_pydict_arrays_and_nested() {
        Python::with_gil(|py| {
            let val = json!({
                "array": [1, "two", false, null, [10, 20]],
                "nested": { "a": 1, "b": [true, false], "c": { "d": null } }
            });

            let obj = json_to_pydict(py, &val).unwrap();
            let dict = obj.downcast_bound::<PyDict>(py).unwrap();

            // Test array
            let any = dict.get_item("array").unwrap().unwrap();
            let array = any.downcast::<PyList>().unwrap();
            assert_eq!(array.len(), 5);
            assert_eq!(array.get_item(0).unwrap().extract::<i64>().unwrap(), 1);
            assert_eq!(array.get_item(1).unwrap().extract::<&str>().unwrap(), "two");
            assert_eq!(array.get_item(2).unwrap().extract::<bool>().unwrap(), false);
            assert!(array.get_item(3).unwrap().is_none());
            let any = array.get_item(4).unwrap().unbind();
            let nested_array = any.downcast_bound::<PyList>(py).unwrap();
            assert_eq!(
                nested_array.get_item(0).unwrap().extract::<i64>().unwrap(),
                10
            );

            // Test nested dict
            let any = dict.get_item("nested").unwrap().unwrap();
            let nested_dict = any.downcast::<PyDict>().unwrap();
            assert_eq!(
                nested_dict
                    .get_item("a")
                    .unwrap()
                    .unwrap()
                    .extract::<i64>()
                    .unwrap(),
                1
            );
            let any = nested_dict.get_item("b").unwrap().unwrap();
            let nested_b = any.downcast::<PyList>().unwrap();
            assert_eq!(
                nested_b.get_item(0).unwrap().extract::<bool>().unwrap(),
                true
            );
            assert_eq!(
                nested_b.get_item(1).unwrap().extract::<bool>().unwrap(),
                false
            );
            let any = nested_dict.get_item("c").unwrap().unwrap();
            let nested_c = any.downcast::<PyDict>().unwrap();
            assert!(nested_c.get_item("d").unwrap().is_none());
        });
    }

    #[test]
    fn test_json_to_pydict_large_numbers() {
        Python::with_gil(|py| {
            let val = json!({
                "small_int": 123,
                "large_int": 1_000_000_000_000_i64,
                "float_num": 1.23456789e10
            });

            let obj = json_to_pydict(py, &val).unwrap();
            let dict = obj.downcast_bound::<PyDict>(py).unwrap();

            assert_eq!(
                dict.get_item("small_int")
                    .unwrap()
                    .unwrap()
                    .extract::<i64>()
                    .unwrap(),
                123
            );
            assert_eq!(
                dict.get_item("large_int")
                    .unwrap()
                    .unwrap()
                    .extract::<i64>()
                    .unwrap(),
                1_000_000_000_000
            );
            assert_eq!(
                dict.get_item("float_num")
                    .unwrap()
                    .unwrap()
                    .extract::<f64>()
                    .unwrap(),
                1.23456789e10
            );
        });
    }

    #[test]
    fn test_json_to_pydict_empty_structures() {
        Python::with_gil(|py| {
            let val = json!({
                "empty_array": [],
                "empty_object": {}
            });

            let obj = json_to_pydict(py, &val).unwrap();
            let dict = obj.downcast_bound::<PyDict>(py).unwrap();

            let any = dict.get_item("empty_array").unwrap().unwrap();
            let empty_array = any.downcast::<PyList>().unwrap();
            assert_eq!(empty_array.len(), 0);
            let any = dict.get_item("empty_object").unwrap().unwrap();
            let empty_object = any.downcast::<PyDict>().unwrap();
            assert_eq!(empty_object.len(), 0);
        });
    }

    #[test]
    fn test_pyglifdata_to_pydict() {
        Python::with_gil(|py| {
            let glif_data = GlifData {
                name: "notehead".into(),
                format: "1".into(),
                format_minor: Some("1".into()),
                advance: Some(Advance {
                    width: Some(100.0),
                    height: Some(120.0),
                }),
                unicodes: vec![0x34, 0x56],
                note: Some("hellow world".into()),
                image: None,
                anchors: vec![],
                guidelines: vec![],
                outline: None,
                lib: None,
            };

            let py_glif = PyGlifData { inner: glif_data };
            let py_obj = py_glif.to_pydict(py).unwrap();
            let dict = py_obj.downcast_bound::<PyDict>(py).unwrap();

            assert_eq!(
                dict.get_item("name")
                    .unwrap()
                    .unwrap()
                    .extract::<&str>()
                    .unwrap(),
                "notehead"
            );
            assert_eq!(
                dict.get_item("width")
                    .unwrap()
                    .unwrap()
                    .extract::<f64>()
                    .unwrap(),
                100.0
            );
            assert_eq!(
                dict.get_item("height")
                    .unwrap()
                    .unwrap()
                    .extract::<f64>()
                    .unwrap(),
                120.0
            );
            assert_eq!(
                dict.get_item("anchors")
                    .unwrap()
                    .unwrap()
                    .downcast::<PyList>()
                    .unwrap()
                    .len(),
                0
            );
            assert_eq!(
                dict.get_item("components")
                    .unwrap()
                    .unwrap()
                    .downcast::<PyList>()
                    .unwrap()
                    .len(),
                0
            );
        });
    }

    #[test]
    fn test_json_to_pydict_bool_edge_cases() {
        Python::with_gil(|py| {
            let val = json!({
                "true_val": true,
                "false_val": false
            });

            let obj = json_to_pydict(py, &val).unwrap();
            let dict = obj.downcast_bound::<PyDict>(py).unwrap();

            assert_eq!(
                dict.get_item("true_val")
                    .unwrap()
                    .unwrap()
                    .extract::<bool>()
                    .unwrap(),
                true
            );
            assert_eq!(
                dict.get_item("false_val")
                    .unwrap()
                    .unwrap()
                    .extract::<bool>()
                    .unwrap(),
                false
            );
        });
    }

    #[test]
    fn test_json_to_pydict_null_and_none() {
        Python::with_gil(|py| {
            let val = json!({
                "null_val": null
            });

            let obj = json_to_pydict(py, &val).unwrap();
            let dict = obj.downcast_bound::<PyDict>(py).unwrap();
            assert!(dict.get_item("null_val").unwrap().is_none());
        });
    }
}
