use std::rc::Rc;

use pyo3::prelude::*;

use crate::bindings::{PyFileProvider, PyProvider};
use crate::font::{Font, Glyph, Iter};

use crate::bindings::PyGlifData;

/// Provide a lazy iterator over `Glyph` objects
#[pyclass(unsendable)]
struct PyIter {
    inner: Iter,
}

#[pymethods]
impl PyIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next(mut slf: PyRefMut<Self>, py: Python) -> Option<Py<PyGlyph>> {
        slf.inner
            .next()
            .map(|glyph| Py::new(py, PyGlyph { inner: glyph }).unwrap())
    }
}

/// Represent the data in a UFO, for lazy glyph creation.
///
/// This class provides a minimal Font object model with dictionary-like features.
/// It stores the `Glyph` objects representing each GLIF file and provides lazy, on-demand data
/// access with minimal load time and memory footprint.
///
/// To use this class, you must provide a concrete implementation of the `Provider` protocol,
/// exemplified by the included `FileProvider` class.
#[pyclass(unsendable, name = "Font")]
pub struct PyFont {
    pub inner: Rc<Font>,
}

#[pymethods]
impl PyFont {
    /// Create a new `Font` by parsing `contents.plist` via the given provider.
    #[new]
    pub fn new(py: Python<'_>, provider: Py<PyAny>) -> PyResult<Self> {
        // Try to downcast to PyFileProvider.
        // Return if Ok. If not, assume it's a custom PyProvider implementation.
        match provider.extract::<PyRef<PyFileProvider>>(py) {
            Ok(file_provider) => {
                let font = Font::new(file_provider.inner.clone())?;
                Ok(Self { inner: font })
            }
            Err(_) => {
                let provider = PyProvider::new(py, provider)?;
                let font = Font::new(provider)?;
                Ok(Self { inner: font })
            }
        }
    }

    /// Return the `Glyph` object mapped to the given name.
    pub fn __getitem__(&self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyGlyph>>> {
        // Ensure the glyph map is initialized.
        Ok(self
            .inner
            .glyph(name)
            .map(|inner_glyph| {
                Py::new(
                    py,
                    PyGlyph {
                        inner: Rc::clone(&inner_glyph),
                    },
                )
            })
            .transpose()?)
    }

    /// Return `True` if the font contains the given name.
    pub fn __contains__(&self, name: &str) -> PyResult<bool> {
        Ok(self.inner.glyph(name).is_some())
    }

    /// Return an iterator over the glyphs in the font.
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<PyIter>> {
        let py = slf.py();
        let iter = slf.inner.iter();
        Py::new(py, PyIter { inner: iter })
    }

    /// Optional: length
    fn __len__(&self) -> usize {
        self.inner.glyphs().len()
    }

    /// Return a list of the glyph names in the font.
    pub fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self
            .inner
            .glyphs()
            .keys()
            .map(|k| k.into())
            .collect::<Vec<_>>())
    }

    /// Return a list of the `Glyph` objects in the font.
    pub fn values(&self, py: Python<'_>) -> PyResult<Vec<Py<PyGlyph>>> {
        let mut vec = Vec::with_capacity(self.inner.contents().len());
        for glyph in self.inner.glyphs().values() {
            let py_glyph = Py::new(
                py,
                PyGlyph {
                    inner: Rc::clone(&glyph),
                },
            )?;
            vec.push(py_glyph);
        }
        Ok(vec)
    }

    /// Return a list of tuples containing each key-value (name, `Glyph`) pair in the font.
    pub fn items(&self, py: Python<'_>) -> PyResult<Vec<(String, Py<PyGlyph>)>> {
        let mut vec = Vec::with_capacity(self.inner.contents().len());
        for (name, glyph) in self.inner.glyphs() {
            let py_glyph = Py::new(
                py,
                PyGlyph {
                    inner: Rc::clone(&glyph),
                },
            )?;
            vec.push((name, py_glyph));
        }
        Ok(vec)
    }
}

/// Represent the data of a UFO GLIF file.
///
/// This class provides it's parent `Font` object with convenient object representations of
/// individual GLIF files. Data attributes are accessible via appropriate property implementations.
/// The actual glyph data is parsed only on demand and cached for efficient reuse.
#[pyclass(unsendable, name = "Glyph")]
pub struct PyGlyph {
    pub inner: Rc<Glyph>,
}

#[pymethods]
impl PyGlyph {
    /// Create a new `Glyph` with the given name in the given font.
    #[new]
    pub fn new(py: Python<'_>, font: PyObject, name: String) -> PyResult<Self> {
        let font = font.extract::<PyRef<PyFont>>(py)?;
        let inner = Glyph::new(Rc::clone(&font.inner), name);

        Ok(Self { inner })
    }

    /// Get the glyph name.
    #[getter]
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Get the major format version string of the glyph's GLIF file.
    #[getter]
    pub fn format(&self) -> PyResult<String> {
        Ok(self.inner.format()?)
    }

    /// Get the minor format version string of the glyph's GLIF file.
    #[getter]
    pub fn format_minor(&self) -> PyResult<Option<String>> {
        Ok(self.inner.format_minor()?)
    }

    /// Get the glyph's advance width/height data (if any).
    #[getter]
    pub fn advance(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let value = self.inner.advance()?;
        Ok(Some(PyGlifData::to_pyobject(py, &value)?))
    }

    /// Get the glyph's list of Unicode code points.
    #[getter]
    pub fn unicodes(&self) -> PyResult<Vec<u32>> {
        Ok(self.inner.unicodes()?)
    }

    /// Get an optional note attached to the glyph.
    #[getter]
    pub fn note(&self) -> PyResult<Option<String>> {
        Ok(self.inner.note()?)
    }

    /// Get the glyph's associated image, if present.
    #[getter]
    pub fn image(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let value = self.inner.image()?;
        Ok(Some(PyGlifData::to_pyobject(py, &value)?))
    }

    /// Get the glyph's guidelines.
    #[getter]
    pub fn guidelines(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let value = self.inner.guidelines()?;
        let data = value
            .into_iter()
            .map(|v| PyGlifData::to_pyobject(py, &v))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(data)
    }

    /// Get the glyph's anchors.
    #[getter]
    pub fn anchors(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let value = self.inner.anchors()?;
        let data = value
            .into_iter()
            .map(|v| PyGlifData::to_pyobject(py, &v))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(data)
    }

    /// Get the glyph's outline, if present.
    #[getter]
    pub fn outline(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let value = self.inner.outline()?;
        Ok(Some(PyGlifData::to_pyobject(py, &value)?))
    }

    /// Get the glyph's lib dictionary (if present).
    #[getter]
    pub fn lib(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let value = self.inner.lib()?;
        Ok(Some(PyGlifData::to_pyobject(py, &value)?))
    }
}
