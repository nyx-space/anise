/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::errors::{AniseError, InternalErrorKind};
use der::{Decode, Encode};
use log::warn;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// A trait to encode / decode ANISE specific data.
pub trait Asn1Serde<'a>: Encode + Decode<'a> {
    /// Saves this context in the providef filename.
    /// If overwrite is set to false, and the filename already exists, this function will return an error.
    ///
    /// TODO: This function should only be available with the alloc feature gate.
    fn save_as(&self, filename: &'a str, overwrite: bool) -> Result<(), AniseError> {
        match self.encoded_len() {
            Err(e) => Err(AniseError::InternalError(e.into())),
            Ok(length) => {
                let len: u32 = length.into();
                // Fill the vector with zeros
                let mut buf = vec![0x0; len as usize];
                self.save_as_via_buffer(filename, overwrite, &mut buf)
            }
        }
    }

    /// Saves this context in the providef filename.
    /// If overwrite is set to false, and the filename already exists, this function will return an error.
    fn save_as_via_buffer(
        &self,
        filename: &'a str,
        overwrite: bool,
        buf: &mut [u8],
    ) -> Result<(), AniseError> {
        if Path::new(filename).exists() {
            if !overwrite {
                return Err(AniseError::FileExists);
            } else {
                warn!("[save_as] overwriting {filename}");
            }
        }

        match File::create(filename) {
            Ok(mut file) => {
                if let Err(e) = self.encode_to_slice(buf) {
                    return Err(InternalErrorKind::Asn1Error(e).into());
                }
                if let Err(e) = file.write_all(buf) {
                    Err(e.kind().into())
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(e.kind().into()),
        }
    }

    /// Attempts to load this data from its bytes
    fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, AniseError> {
        match Self::from_der(bytes) {
            Ok(yay) => Ok(yay),
            Err(e) => Err(AniseError::DecodingError(e)),
        }
    }
}
