/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::{debug, info, warn};
use platform_dirs::AppDirs;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use serde_dhall::StaticType;
use std::env;
use std::fs::{create_dir_all, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::thread;
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

/// MetaFile allows downloading a remote file from a URL (http, https only), and interpolation of paths in environment variable using the Dhall syntax `env:MY_ENV_VAR`.
///
/// The data is stored in the user's local temp directory (i.e. `~/.local/share/nyx-space/anise/` on Linux and `AppData/Local/nyx-space/anise/` on Windows).
/// Prior to loading a remote resource, if the local resource exists, its CRC32 will be computed: if it matches the CRC32 of this instance of MetaFile,
/// then the file will not be downloaded a second time.
///
/// :type uri: str
/// :type crc32: int, optional
/// :rtype: MetaFile
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, StaticType)]
pub struct MetaFile {
    /// URI of this meta file
    pub uri: String,
    /// Optionally specify the CRC32 of this file, which will be checked prior to loading.
    pub crc32: Option<u32>,
}

impl MetaFile {
    /// Processes this MetaFile by downloading it if it's a URL and sets this structure's `uri` field to the local path
    ///
    /// This function modified `self` and changes the URI to be the path to the downloaded file.
    #[cfg(not(feature = "python"))]
    pub fn process(&mut self, autodelete: bool) -> Result<(), MetaAlmanacError> {
        self._process(autodelete)
    }

    pub(crate) fn _process(&mut self, autodelete: bool) -> Result<(), MetaAlmanacError> {
        // First, parse environment variables if any.
        self.uri = replace_env_vars(&self.uri);
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

                                        if !app_dir.data_dir.exists() {
                                            // Create the folders
                                            create_dir_all(&app_dir.data_dir).map_err(|e| {
                                                MetaAlmanacError::MetaIO {
                                                    path: app_dir.data_dir.to_str().unwrap().into(),
                                                    what: "creating directories for storage",
                                                    source: InputOutputError::IOError {
                                                        kind: e.kind(),
                                                    },
                                                }
                                            })?;
                                        }

                                        let dest_path = app_dir.data_dir.join(file_name);
                                        let lock_path = dest_path.with_file_name(
                                            file_name.to_str().unwrap().to_string() + ".lock",
                                        );

                                        // Check the existence of the lock file.
                                        let mut checks = 0;
                                        loop {
                                            if lock_path.exists() {
                                                if checks == 9 {
                                                    if autodelete {
                                                        info!(
                                                            "deleting lock file {}",
                                                            dest_path.to_str().unwrap().to_owned()
                                                        );
                                                        if let Err(e) = remove_file(&lock_path) {
                                                            warn!("{e} -- ignoring");
                                                        }
                                                        break;
                                                    } else {
                                                        return Err(
                                                            MetaAlmanacError::PersistentLock {
                                                                desired: dest_path
                                                                    .to_str()
                                                                    .unwrap()
                                                                    .to_owned(),
                                                            },
                                                        );
                                                    }
                                                }

                                                checks += 1;
                                                thread::sleep(std::time::Duration::from_secs(1));
                                            } else {
                                                break;
                                            }
                                        }

                                        if dest_path.exists() {
                                            if let Some(crc32) = self.crc32 {
                                                // Open the file and check the CRC32
                                                let dest_path_c = dest_path.clone(); // macro token issue
                                                if let Ok(bytes) = file2heap!(dest_path_c) {
                                                    let computed_crc32 = crc32fast::hash(&bytes);
                                                    let dest_path_s =
                                                        dest_path.to_str().unwrap().to_string();
                                                    if computed_crc32 == crc32 {
                                                        // No need to redownload this, let's just update the uri path
                                                        info!("Using cached {dest_path_s}",);
                                                        self.uri = dest_path_s;
                                                        return Ok(());
                                                    } else {
                                                        info!("Discarding cached {dest_path_s} - CRC32 differ (got {computed_crc32:x}, expected {crc32:x})");
                                                    }
                                                }
                                            }
                                        }

                                        // At this stage, either the dest path does not exist, or the CRC32 check failed.

                                        // Create the lock file
                                        if let Err(e) = File::create(&lock_path) {
                                            return Err(MetaAlmanacError::MetaIO {
                                                path: dest_path
                                                    .join(".lock")
                                                    .to_str()
                                                    .unwrap()
                                                    .into(),
                                                what: "creating lock file",
                                                source: InputOutputError::IOError {
                                                    kind: e.kind(),
                                                },
                                            });
                                        }

                                        let del_lock_file = || {
                                            // Ignore if the deletion of the lock file fails
                                            let _ = remove_file(lock_path);
                                        };

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
                                                        Err(e) => {
                                                            del_lock_file();
                                                            Err(MetaAlmanacError::MetaIO {
                                                                path: dest_path
                                                                    .to_str()
                                                                    .unwrap()
                                                                    .into(),
                                                                what: "creating file for storage",
                                                                source: InputOutputError::IOError {
                                                                    kind: e.kind(),
                                                                },
                                                            })
                                                        }
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

                                                            del_lock_file();

                                                            Ok(())
                                                        }
                                                    }
                                                } else {
                                                    del_lock_file();
                                                    Err(MetaAlmanacError::FetchError {
                                                        status: resp.status(),
                                                        uri: self.uri.clone(),
                                                    })
                                                }
                                            }
                                            Err(e) => {
                                                del_lock_file();
                                                Err(MetaAlmanacError::CnxError {
                                                    uri: self.uri.clone(),
                                                    error: format!("{e}"),
                                                })
                                            }
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
    ///
    /// :type autodelete: bool, optional
    /// :rtype: None
    pub fn process(
        &mut self,
        py: Python,
        autodelete: Option<bool>,
    ) -> Result<(), MetaAlmanacError> {
        py.allow_threads(|| self._process(autodelete.unwrap_or(false)))
    }

    /// :rtype: str
    #[getter]
    fn get_uri(&self) -> PyResult<String> {
        Ok(self.uri.clone())
    }
    /// :type uri: str
    #[setter]
    fn set_uri(&mut self, uri: String) -> PyResult<()> {
        self.uri = uri;
        Ok(())
    }
    /// :rtype: int
    #[getter]
    fn get_crc32(&self) -> PyResult<Option<u32>> {
        Ok(self.crc32)
    }
    /// :type crc32: int
    #[setter]
    fn set_crc32(&mut self, crc32: Option<u32>) -> PyResult<()> {
        self.crc32 = crc32;
        Ok(())
    }
}

fn replace_env_vars(input: &str) -> String {
    let re = Regex::new(r"env:([A-Z_][A-Z0-9_]*)").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let var_name = &caps[1];
        env::var(var_name).unwrap_or_else(|_| format!("env:{}", var_name))
    })
    .to_string()
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
        assert!(window_path._process(true).is_ok());
        assert_eq!(window_path.uri, "C:\\Users\\me\\meta.dhall".to_string());

        let mut file_prefix_path = MetaFile {
            uri: "fIlE:///Users/me/meta.dhall".to_string(),
            crc32: None,
        };
        assert!(file_prefix_path._process(true).is_ok());
        assert_eq!(file_prefix_path.uri, "/Users/me/meta.dhall".to_string());

        let mut unix_abs_path = MetaFile {
            uri: "/Users/me/meta.dhall".to_string(),
            crc32: None,
        };
        assert!(unix_abs_path._process(true).is_ok());
        assert_eq!(unix_abs_path.uri, "/Users/me/meta.dhall".to_string());

        let mut unix_rel_path = MetaFile {
            uri: "../Users/me/meta.dhall".to_string(),
            crc32: None,
        };
        assert!(unix_rel_path._process(true).is_ok());
        assert_eq!(unix_rel_path.uri, "../Users/me/meta.dhall".to_string());
    }

    #[test]
    fn test_metafile_regex() {
        use std::env;
        let mut user_path = MetaFile {
            uri: "env:USER/.cargo/env".to_string(),
            crc32: None,
        };
        user_path._process(false).unwrap();
        assert_eq!(user_path.uri, env::var("USER").unwrap() + "/.cargo/env");

        let mut unknown_path = MetaFile {
            uri: "env:BLAH_BLAH_NO_EXIST/.cargo/env".to_string(),
            crc32: None,
        };
        unknown_path._process(false).unwrap();
        assert_eq!(
            unknown_path.uri,
            "env:BLAH_BLAH_NO_EXIST/.cargo/env".to_string()
        );
    }
}
