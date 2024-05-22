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

use serde_derive::{Deserialize, Serialize};
use serde_dhall::StaticType;
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

use crate::file2heap;
use crate::prelude::InputOutputError;

use super::MetaAlmanacError;

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

impl MetaFile {
    /// Processes this MetaFile by downloading it if it's a URL.
    ///
    /// This function modified `self` and changes the URI to be the path to the downloaded file.
    #[cfg(not(feature = "python"))]
    pub fn process(&mut self) -> Result<(), MetaAlmanacError> {
        self._process()
    }

    pub(crate) fn _process(&mut self) -> Result<(), MetaAlmanacError> {
        match Url::parse(&self.uri) {
            Err(e) => {
                debug!("parsing {} caused {e} -- assuming local path", self.uri);
                Ok(())
            }
            Ok(url) => {
                if !url.scheme().starts_with("http") {
                    // This means it could be either a path with `file:///`, or an absolute path on Windows.
                    if url.scheme() == "file" {
                        // Remove the first four characters plus `://`, regardless of case
                        self.uri = self.uri[7..].to_string();
                    }
                    return Ok(());
                }
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
                                                    Err(MetaAlmanacError::FetchError {
                                                        status: resp.status(),
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

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl MetaFile {
    /// Builds a new MetaFile from the provided URI and optionally its CRC32 checksum.
    #[new]
    pub fn py_new(uri: String, crc32: Option<u32>) -> Self {
        Self { uri, crc32 }
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

    /// Processes this MetaFile by downloading it if it's a URL.
    ///
    /// This function modified `self` and changes the URI to be the path to the downloaded file.
    pub fn process(&mut self, py: Python) -> Result<(), MetaAlmanacError> {
        py.allow_threads(|| self._process())
    }
}

#[cfg(test)]
mod ut_metafile {
    use super::MetaFile;

    #[test]
    fn abs_paths() {
        let mut window_path = MetaFile {
            uri: "C:\\Users\\me\\meta.dhall".to_string(),
            crc32: None,
        };
        assert!(window_path._process().is_ok());
        assert_eq!(window_path.uri, "C:\\Users\\me\\meta.dhall".to_string());

        let mut file_prefix_path = MetaFile {
            uri: "fIlE:///Users/me/meta.dhall".to_string(),
            crc32: None,
        };
        assert!(file_prefix_path._process().is_ok());
        assert_eq!(file_prefix_path.uri, "/Users/me/meta.dhall".to_string());

        let mut unix_abs_path = MetaFile {
            uri: "/Users/me/meta.dhall".to_string(),
            crc32: None,
        };
        assert!(unix_abs_path._process().is_ok());
        assert_eq!(unix_abs_path.uri, "/Users/me/meta.dhall".to_string());

        let mut unix_rel_path = MetaFile {
            uri: "../Users/me/meta.dhall".to_string(),
            crc32: None,
        };
        assert!(unix_rel_path._process().is_ok());
        assert_eq!(unix_rel_path.uri, "../Users/me/meta.dhall".to_string());
    }
}
