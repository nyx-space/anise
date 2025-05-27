use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::PyType;

use std::str::FromStr;

use crate::errors::AlmanacResult;

use super::{Almanac, MetaAlmanac, MetaAlmanacError, MetaFile};

// Python only methods
#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl MetaAlmanac {
    /// Loads the provided path as a Dhall file. If no path is provided, creates an empty MetaAlmanac that can store MetaFiles.
    #[new]
    #[pyo3(signature=(maybe_path=None))]
    pub fn py_new(maybe_path: Option<String>) -> Result<Self, MetaAlmanacError> {
        match maybe_path {
            Some(path) => Self::new(path),
            None => Ok(Self { files: Vec::new() }),
        }
    }

    /// Loads the provided string as a Dhall configuration to build a MetaAlmanac
    ///
    /// :type s: str
    /// :rtype: MetaAlmanac
    #[classmethod]
    fn loads(_cls: &Bound<'_, PyType>, s: String) -> Result<Self, MetaAlmanacError> {
        Self::from_str(&s)
    }

    /// Returns an Almanac loaded from the latest NAIF data via the `default` MetaAlmanac.
    /// The MetaAlmanac will download the DE440s.bsp file, the PCK0008.PCA, the full Moon Principal Axis BPC (moon_pa_de440_200625) and the latest high precision Earth kernel from JPL.
    ///
    /// # File list
    /// - <http://public-data.nyxspace.com/anise/de440s.bsp>
    /// - <http://public-data.nyxspace.com/anise/v0.5/pck08.pca>
    /// - <http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc>
    /// - <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc>
    ///
    /// # Reproducibility
    ///
    /// Note that the `earth_latest_high_prec.bpc` file is regularly updated daily (or so). As such,
    /// if queried at some future time, the Earth rotation parameters may have changed between two queries.
    ///
    /// Set `autodelete` to true to delete lock file if a dead lock is detected after 10 seconds.
    ///
    /// :type autodelete: bool, optional
    /// :rtype: MetaAlmanac
    #[classmethod]
    #[pyo3(name = "latest")]
    #[pyo3(signature=(autodelete=None))]
    fn py_latest(
        _cls: &Bound<'_, PyType>,
        py: Python,
        autodelete: Option<bool>,
    ) -> AlmanacResult<Almanac> {
        let mut meta = Self::default();
        py.allow_threads(|| match meta.process(autodelete.unwrap_or(false)) {
            Ok(almanac) => Ok(almanac),
            Err(e) => Err(e),
        })
    }

    /// Fetch all of the URIs and return a loaded Almanac.
    /// When downloading the data, ANISE will create a temporarily lock file to prevent race conditions
    /// where multiple processes download the data at the same time. Set `autodelete` to true to delete
    /// this lock file if a dead lock is detected after 10 seconds. Set this flag to false if you have
    /// more than ten processes which may attempt to download files in parallel.
    ///
    /// :type autodelete: bool, optional
    /// :rtype: Almanac
    #[pyo3(name = "process")]
    #[pyo3(signature=(autodelete=None))]
    pub fn py_process(&mut self, py: Python, autodelete: Option<bool>) -> AlmanacResult<Almanac> {
        py.allow_threads(|| self.process(autodelete.unwrap_or(true)))
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    fn __repr__(&self) -> String {
        format!("{self:?} (@{self:p})")
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }

    /// :rtype: typing.List
    #[getter]
    fn get_files(&self) -> PyResult<Vec<MetaFile>> {
        Ok(self.files.clone())
    }
    /// :type files: typing.List
    #[setter]
    fn set_files(&mut self, files: Vec<MetaFile>) -> PyResult<()> {
        self.files = files;
        Ok(())
    }
}
