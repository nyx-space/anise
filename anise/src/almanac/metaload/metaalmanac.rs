/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use serde_derive::{Deserialize, Serialize};
use serde_dhall::SimpleType;
use snafu::prelude::*;
use std::str::FromStr;
use url::Url;

#[cfg(feature = "python")]
use pyo3::exceptions::PyTypeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;
#[cfg(feature = "python")]
use pyo3::types::PyType;

use crate::errors::{AlmanacResult, MetaSnafu};

use super::{Almanac, MetaAlmanacError, MetaFile};

/// A structure to set up an Almanac, with automatic downloading, local storage, checksum checking, and more.
///
/// # Behavior
/// If the URI is a local path, relative or absolute, nothing will be fetched from a remote. Relative paths are relative to the execution folder (i.e. the current working directory).
/// If the URI is a remote path, the MetaAlmanac will first check if the file exists locally. If it exists, it will check that the CRC32 checksum of this file matches that of the specs.
/// If it does not match, the file will be downloaded again. If no CRC32 is provided but the file exists, then the MetaAlmanac will fetch the remote file and overwrite the existing file.
/// The downloaded path will be stored in the "AppData" folder.
///
/// :type maybe_path: str, optional
/// :rtype: MetaAlmanac
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct MetaAlmanac {
    pub files: Vec<MetaFile>,
}

impl MetaAlmanac {
    /// Loads the provided path as a Dhall configuration file and processes each file.
    pub fn new(path: String) -> Result<Self, MetaAlmanacError> {
        match serde_dhall::from_file(&path).parse::<Self>() {
            Err(e) => Err(MetaAlmanacError::ParseDhall {
                path,
                err: format!("{e}"),
            }),
            Ok(me) => Ok(me),
        }
    }

    /// Fetch all of the URIs and return a loaded Almanac
    pub(crate) fn _process(&mut self, autodelete: bool) -> AlmanacResult<Almanac> {
        for (fno, file) in self.files.iter_mut().enumerate() {
            file._process(autodelete).context(MetaSnafu {
                fno,
                file: file.clone(),
            })?;
        }
        // At this stage, all of the files are local files, so we can load them as is.
        let mut ctx = Almanac::default();
        for uri in &self.files {
            ctx = ctx.load(&uri.uri)?;
        }
        Ok(ctx)
    }

    /// Fetch all of the URIs and return a loaded Almanac
    /// When downloading the data, ANISE will create a temporarily lock file to prevent race conditions
    /// where multiple processes download the data at the same time. Set `autodelete` to true to delete
    /// this lock file if a dead lock is detected after 10 seconds. Set this flag to false if you have
    /// more than ten processes which may attempt to download files in parallel.
    #[cfg(not(feature = "python"))]
    pub fn process(&mut self, autodelete: bool) -> AlmanacResult<Almanac> {
        self._process(autodelete)
    }

    /// Returns an Almanac loaded from the latest NAIF data via the `default` MetaAlmanac.
    /// The MetaAlmanac will download the DE440s.bsp file, the PCK0008.PCA, the full Moon Principal Axis BPC (moon_pa_de440_200625) and the latest high precision Earth kernel from JPL.
    ///
    /// # File list
    /// - <http://public-data.nyxspace.com/anise/de440s.bsp>
    /// - <http://public-data.nyxspace.com/anise/v0.5/pck11.pca>
    /// - <http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc>
    /// - <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc>
    ///
    /// # Reproducibility
    ///
    /// Note that the `earth_latest_high_prec.bpc` file is regularly updated daily (or so). As such,
    /// if queried at some future time, the Earth rotation parameters may have changed between two queries.
    #[cfg(not(feature = "python"))]
    pub fn latest() -> AlmanacResult<Almanac> {
        Self::default().process(true)
    }
}

impl FromStr for MetaAlmanac {
    type Err = MetaAlmanacError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_dhall::from_str(s).parse::<Self>() {
            Err(e) => Err(MetaAlmanacError::ParseDhall {
                path: s.to_string(),
                err: format!("{e}"),
            }),
            Ok(me) => Ok(me),
        }
    }
}

// Methods shared between Rust and Python
#[cfg_attr(feature = "python", pymethods)]
#[allow(deprecated_in_future)]
impl MetaAlmanac {
    /// Dumps the configured Meta Almanac into a Dhall string.
    ///
    /// :rtype: str
    pub fn dumps(&self) -> Result<String, MetaAlmanacError> {
        // Define the Dhall type
        let dhall_type: SimpleType =
            serde_dhall::from_str("{ files : List { uri : Text, crc32 : Optional Natural } }")
                .parse()
                .unwrap();

        serde_dhall::serialize(&self)
            .type_annotation(&dhall_type)
            .to_string()
            .map_err(|e| MetaAlmanacError::ExportDhall {
                err: format!("{e}"),
            })
    }
}

// Python only methods
#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl MetaAlmanac {
    /// Loads the provided path as a Dhall file. If no path is provided, creates an empty MetaAlmanac that can store MetaFiles.
    #[new]
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
    fn latest(
        _cls: &Bound<'_, PyType>,
        py: Python,
        autodelete: Option<bool>,
    ) -> AlmanacResult<Almanac> {
        let mut meta = Self::default();
        py.allow_threads(|| match meta._process(autodelete.unwrap_or(false)) {
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
    pub fn process(&mut self, py: Python, autodelete: Option<bool>) -> AlmanacResult<Almanac> {
        py.allow_threads(|| self._process(autodelete.unwrap_or(true)))
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

/// By default, the MetaAlmanac will download the DE440s.bsp file, the PCK0008.PCA, the full Moon Principal Axis BPC (moon_pa_de440_200625) and the latest high precision Earth kernel from JPL.
///
/// # File list
/// - <http://public-data.nyxspace.com/anise/de440s.bsp>
/// - <http://public-data.nyxspace.com/anise/v0.5/pck11.pca>
/// - <http://public-data.nyxspace.com/anise/v0.5/moon_fk.epa>
/// - <http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc>
/// - <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc>
///
/// # Reproducibility
///
/// Note that the `earth_latest_high_prec.bpc` file is regularly updated daily (or so). As such,
/// if queried at some future time, the Earth rotation parameters may have changed between two queries.
///
impl Default for MetaAlmanac {
    fn default() -> Self {
        let nyx_cloud_stor = Url::parse("http://public-data.nyxspace.com/anise/").unwrap();
        let jpl_cloud_stor =
            Url::parse("https://naif.jpl.nasa.gov/pub/naif/generic_kernels/").unwrap();

        Self {
            files: vec![
                MetaFile {
                    uri: nyx_cloud_stor.join("de440s.bsp").unwrap().to_string(),
                    crc32: Some(0x7286750a),
                },
                MetaFile {
                    uri: nyx_cloud_stor.join("v0.5/pck11.pca").unwrap().to_string(),
                    crc32: Some(0x8213b6e9),
                },
                MetaFile {
                    uri: nyx_cloud_stor.join("v0.5/moon_fk.epa").unwrap().to_string(),
                    crc32: Some(0xb93ba21),
                },
                MetaFile {
                    uri: nyx_cloud_stor
                        .join("moon_pa_de440_200625.bpc")
                        .unwrap()
                        .to_string(),
                    crc32: Some(0xcde5ca7d),
                },
                MetaFile {
                    uri: jpl_cloud_stor
                        .join("pck/earth_latest_high_prec.bpc")
                        .unwrap()
                        .to_string(),
                    crc32: None,
                },
            ],
        }
    }
}
