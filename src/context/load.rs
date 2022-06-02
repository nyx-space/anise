/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use memmap2::Mmap;

use crate::der::Decode;
use crate::log::{error, trace};
use crate::{
    asn1::{context::AniseContext, semver::Semver, ANISE_VERSION},
    errors::AniseError,
};

impl<'a> AniseContext<'a> {
    /// Try to load an Anise file from a pointer of bytes
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, AniseError> {
        match Self::from_der(bytes) {
            Ok(ctx) => {
                trace!("[try_from_bytes] loaded context successfully");
                Ok(ctx)
            }
            Err(e) => {
                // If we can't load the file, let's try to load the version only to be helpful
                match Semver::from_der(&bytes[0..5]) {
                    Ok(file_version) => {
                        if file_version == ANISE_VERSION {
                            error!("[try_from_bytes] context bytes corrupted but ANISE library version match");
                            Err(AniseError::DecodingError(e))
                        } else {
                            error!(
                                "[try_from_bytes] context bytes and ANISE library version mismatch"
                            );
                            Err(AniseError::IncompatibleVersion {
                                got: file_version,
                                exp: ANISE_VERSION,
                            })
                        }
                    }
                    Err(_) => {
                        error!("[try_from_bytes] context bytes not in ANISE format");
                        Err(AniseError::DecodingError(e))
                    }
                }
            }
        }
    }

    /// Forces to load an Anise file from a pointer of bytes.
    /// **Panics** if the bytes cannot be interpreted as an Anise file.
    pub fn from_bytes(buf: &'a [u8]) -> Self {
        Self::try_from_bytes(buf).unwrap()
    }
}

impl<'a> TryFrom<&'a Mmap> for AniseContext<'a> {
    type Error = AniseError;

    fn try_from(buf: &'a Mmap) -> Result<Self, Self::Error> {
        Self::try_from_bytes(buf)
    }
}
