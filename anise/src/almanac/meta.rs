/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::{debug, info};
use platform_dirs::AppDirs;
use reqwest::StatusCode;
use serde_derive::{Deserialize, Serialize};
use serde_dhall::{SimpleType, StaticType};
use snafu::prelude::*;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use url::Url;

#[cfg(feature = "python")]
use pyo3::exceptions::PyTypeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;

use crate::errors::{AlmanacError, MetaSnafu};
use crate::file2heap;
use crate::prelude::InputOutputError;

use super::Almanac;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum MetaAlmanacError {
    #[snafu(display("could not create the cache folder for ANISE, please use a relative path"))]
    AppDirError,
    #[snafu(display("could not find a file path in {path}"))]
    MissingFilePath { path: String },
    #[snafu(display("IO error {source} when {what} with {path}"))]
    MetaIO {
        path: String,
        what: &'static str,
        source: InputOutputError,
    },
    #[snafu(display("fetching {uri} returned {status}"))]
    FetchError { status: StatusCode, uri: String },
    #[snafu(display("connection {uri} returned {error}"))]
    CnxError { uri: String, error: String },
    #[snafu(display("error parsing {path} as Dhall config: {err}"))]
    ParseDhall { path: String, err: String },
    #[snafu(display("error exporting as Dhall config: {err}"))]
    ExportDhall { err: String },
}

/// A structure to set up an Almanac, with automatic downloading, local storage, checksum checking, and more.
///
/// # Behavior
/// If the URI is a local path, relative or absolute, nothing will be fetched from a remote. Relative paths are relative to the execution folder (i.e. the current working directory).
/// If the URI is a remote path, the MetaAlmanac will first check if the file exists locally. If it exists, it will check that the CRC32 checksum of this file matches that of the specs.
/// If it does not match, the file will be downloaded again. If no CRC32 is provided but the file exists, then the MetaAlmanac will fetch the remote file and overwrite the existing file.
/// The downloaded path will be stored in the "AppData" folder.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
#[cfg_attr(feature = "python", pyo3(get_all, set_all))]
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
}

#[cfg_attr(feature = "python", pymethods)]
impl MetaAlmanac {
    /// Loads the provided path as a Dhall file. If no path is provided, creates an empty MetaAlmanac that can store MetaFiles.
    #[cfg(feature = "python")]
    #[new]
    pub fn py_new(maybe_path: Option<String>) -> Result<Self, MetaAlmanacError> {
        match maybe_path {
            Some(path) => Self::new(path),
            None => Ok(Self { files: Vec::new() }),
        }
    }

    /// Fetch all of the data and return a loaded Almanac
    pub fn process(&mut self) -> Result<Almanac, AlmanacError> {
        for uri in &mut self.files {
            uri.process().with_context(|_| MetaSnafu)?;
        }
        // At this stage, all of the files are local files, so we can load them as is.
        let mut ctx = Almanac::default();
        for uri in &self.files {
            ctx = ctx.load(&uri.uri)?;
        }
        Ok(ctx)
    }

    /// Dumps the configured Meta Almanac into a Dhall string
    pub fn dump(&self) -> Result<String, MetaAlmanacError> {
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

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self:?} (@{self:p})")
    }

    #[cfg(feature = "python")]
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }
}

/// By default, the MetaAlmanac will download the DE440s.bsp file, the PCK0008.PCA, and the latest high precision Earth kernel from JPL.
///
/// # File list
/// - <http://public-data.nyxspace.com/anise/de440s.bsp>
/// - <http://public-data.nyxspace.com/anise/pck08.pca>
/// - <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc>
///
/// # Reproducibility
///
/// Note that the `earth_latest_high_prec.bpc` file is regularily updated daily (or so). As such,
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
                    uri: nyx_cloud_stor.join("pck08.pca").unwrap().to_string(),
                    crc32: Some(0x487bee78),
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

#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
#[cfg_attr(feature = "python", pyo3(get_all, set_all))]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, StaticType)]
pub struct MetaFile {
    /// URI of this meta file
    pub uri: String,
    /// Optionally specify the CRC32 of this file, which will be checked prior to loading.
    pub crc32: Option<u32>,
}

#[cfg_attr(feature = "python", pymethods)]
impl MetaFile {
    /// Builds a new MetaFile from the provided URI and optionally its CRC32 checksum.
    #[cfg(feature = "python")]
    #[new]
    pub fn py_new(uri: String, crc32: Option<u32>) -> Self {
        Self { uri, crc32 }
    }

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self:?} (@{self:p})")
    }

    #[cfg(feature = "python")]
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }

    /// Processes this MetaFile by downloading it if it's a URL.
    ///
    /// This function modified `self` and changes the URI to be the path to the downloaded file.
    fn process(&mut self) -> Result<(), MetaAlmanacError> {
        match Url::parse(&self.uri) {
            Err(e) => {
                debug!("parsing {} caused {e} -- assuming local path", self.uri);
                Ok(())
            }
            Ok(url) => {
                // Build the path for this file.
                match url.path_segments().and_then(|segments| segments.last()) {
                    Some(remote_file_path) => {
                        match Path::new(remote_file_path).file_name() {
                            Some(file_name) => {
                                match AppDirs::new(Some("nyx-space/anise"), true) {
                                    Some(app_dir) => {
                                        // Check whether the path currently exists.
                                        let dest_path = app_dir.data_dir.join(file_name);

                                        if !app_dir.data_dir.exists() {
                                            // Create the folders
                                            create_dir_all(app_dir.data_dir).map_err(|e| {
                                                MetaAlmanacError::MetaIO {
                                                    path: dest_path.to_str().unwrap().into(),
                                                    what: "creating directories for storage",
                                                    source: InputOutputError::IOError {
                                                        kind: e.kind(),
                                                    },
                                                }
                                            })?;
                                        }

                                        if dest_path.exists() {
                                            if let Some(crc32) = self.crc32 {
                                                // Open the file and check the CRC32
                                                let dest_path_c = dest_path.clone(); // macro token issue
                                                if let Ok(bytes) = file2heap!(dest_path_c) {
                                                    if crc32fast::hash(&bytes) == crc32 {
                                                        // No need to redownload this, let's just update the uri path
                                                        self.uri =
                                                            dest_path.to_str().unwrap().to_string();
                                                        info!(
                                                            "Using cached {} (CRC32 matched)",
                                                            self.uri
                                                        );
                                                        return Ok(());
                                                    }
                                                }
                                            }
                                        }

                                        // At this stage, either the dest path does not exist, or the CRC32 check failed.
                                        let client = reqwest::blocking::Client::builder()
                                            .connect_timeout(Duration::from_secs(30))
                                            .timeout(Duration::from_secs(30))
                                            .build()
                                            .unwrap();

                                        match client.get(url.clone()).send() {
                                            Ok(resp) => {
                                                if resp.status().is_success() {
                                                    // Downloaded the file, let's store it locally.
                                                    match File::create(&dest_path) {
                                                        Err(e) => Err(MetaAlmanacError::MetaIO {
                                                            path: dest_path
                                                                .to_str()
                                                                .unwrap()
                                                                .into(),
                                                            what: "creating file for storage",
                                                            source: InputOutputError::IOError {
                                                                kind: e.kind(),
                                                            },
                                                        }),
                                                        Ok(mut file) => {
                                                            // Created the file, let's write the bytes.
                                                            let bytes = resp.bytes().unwrap();
                                                            let crc32 = crc32fast::hash(&bytes);
                                                            file.write_all(&bytes).unwrap();

                                                            info!(
                                                                "Saved {url} to {} (CRC32 = {crc32:x})",
                                                                dest_path.to_str().unwrap()
                                                            );

                                                            // Set the URI for loading
                                                            self.uri = dest_path
                                                                .to_str()
                                                                .unwrap()
                                                                .to_string();

                                                            // Set the CRC32
                                                            self.crc32 = Some(crc32);

                                                            Ok(())
                                                        }
                                                    }
                                                } else {
                                                    println!("err");
                                                    let err = resp.error_for_status().unwrap();
                                                    Err(MetaAlmanacError::FetchError {
                                                        status: err.status(),
                                                        uri: self.uri.clone(),
                                                    })
                                                }
                                            }
                                            Err(e) => Err(MetaAlmanacError::CnxError {
                                                uri: self.uri.clone(),
                                                error: format!("{e}"),
                                            }),
                                        }
                                    }
                                    None => Err(MetaAlmanacError::AppDirError),
                                }
                            }
                            None => Err(MetaAlmanacError::MissingFilePath {
                                path: self.uri.clone(),
                            }),
                        }
                    }
                    None => Err(MetaAlmanacError::MissingFilePath {
                        path: self.uri.clone(),
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod meta_test {
    use super::{MetaAlmanac, Path};
    use std::env;

    #[test]
    fn test_meta_almanac() {
        let _ = pretty_env_logger::try_init();
        let mut meta = MetaAlmanac::default();
        println!("{meta:?}");

        let almanac = meta.process().unwrap();
        println!("{almanac}");

        // Process again to confirm that the CRC check works
        assert!(meta.process().is_ok());
    }

    #[test]
    fn test_from_dhall() {
        let default = MetaAlmanac::default();

        println!("{}", default.dump().unwrap());

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/default_meta.dhall");
        let dhall = MetaAlmanac::new(path.to_str().unwrap().to_string()).unwrap();

        assert_eq!(dhall, default);
    }
}
