/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::{marker::PhantomData, ops::Deref};

use bytes::BytesMut;
use snafu::ResultExt;
use zerocopy::AsBytes;

use crate::{
    errors::{DecodingError, InputOutputError},
    file2heap,
};

use super::{
    daf::MutDAF, DAFError, DecodingNameSnafu, IOSnafu, NAIFSummaryRecord, NameRecord, RCRD_LEN,
};

macro_rules! io_imports {
    () => {
        use std::fs::File;
    };
}

io_imports!();

impl<R: NAIFSummaryRecord> MutDAF<R> {
    /// Parse the provided bytes as a SPICE Double Array File
    pub fn parse<B: Deref<Target = [u8]>>(bytes: B) -> Result<Self, DAFError> {
        let crc32_checksum = crc32fast::hash(&bytes);
        let mut buf = BytesMut::with_capacity(0);
        buf.extend(bytes.iter());
        let me = Self {
            bytes: buf,
            crc32_checksum,
            _daf_type: PhantomData,
        };
        // Check that these calls will succeed.
        me.file_record()?;
        me.name_record()?;
        Ok(me)
    }

    pub fn load(path: &str) -> Result<Self, DAFError> {
        let bytes = file2heap!(path).with_context(|_| IOSnafu {
            action: format!("loading {path:?}"),
        })?;

        Self::parse(bytes)
    }

    /// Sets the name record of this mutable DAF file to the one provided as a parameter.
    pub fn set_name_record(&mut self, new_name_record: NameRecord) -> Result<(), DAFError> {
        let rcrd_idx = self.file_record()?.fwrd_idx() * RCRD_LEN;
        let size = self.bytes.len();
        let rcrd_bytes = self
            .bytes
            .get_mut(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| DecodingError::InaccessibleBytes {
                start: rcrd_idx,
                end: rcrd_idx + RCRD_LEN,
                size,
            })
            .with_context(|_| DecodingNameSnafu { kind: R::NAME })?;
        rcrd_bytes.copy_from_slice(new_name_record.as_bytes());
        Ok(())
    }
}
