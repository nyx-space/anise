/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crate::{
    structure::lookuptable::{Entry, LutError},
    NaifId,
};
use bytes::Bytes;
use snafu::prelude::*;

use super::{
    error::{DataSetError, DataSetLutSnafu},
    DataSet, DataSetT,
};

/// Dataset builder allows building a dataset. It requires allocations.
#[derive(Clone, Default, Debug)]
pub struct DataSetBuilder<T: DataSetT, const ENTRIES: usize> {
    pub dataset: DataSet<T, ENTRIES>,
}

impl<'a, T: DataSetT, const ENTRIES: usize> DataSetBuilder<T, ENTRIES> {
    pub fn push_into(
        &mut self,
        buf: &mut Vec<u8>,
        data: &T,
        id: Option<NaifId>,
        name: Option<&'a str>,
    ) -> Result<(), DataSetError> {
        let mut this_buf = vec![];
        data.encode_to_vec(&mut this_buf).unwrap();
        // Build this entry data.
        let entry = Entry {
            start_idx: buf.len() as u32,
            end_idx: (buf.len() + this_buf.len()) as u32,
        };

        match id {
            Some(id) => {
                match name {
                    Some(name) => {
                        // Both an ID and a name
                        self.dataset.lut.append(id, name, entry).with_context(|_| {
                            DataSetLutSnafu {
                                action: "pushing data with ID and name",
                            }
                        })?;
                        // If the ID is the body of a system with a single object, also insert it for the system ID.
                        if [199, 299].contains(&id) {
                            self.dataset
                                .lut
                                .append(id / 100, name, entry)
                                .with_context(|_| DataSetLutSnafu {
                                    action: "pushing data with ID and name",
                                })?;
                        }
                    }
                    None => {
                        // Only an ID and no name
                        self.dataset.lut.append_id(id, entry).with_context(|_| {
                            DataSetLutSnafu {
                                action: "pushing data with ID only",
                            }
                        })?;
                        // If the ID is the body of a system with a single object, also insert it for the system ID.
                        if [199, 299].contains(&id) {
                            self.dataset
                                .lut
                                .append_id(id / 100, entry)
                                .with_context(|_| DataSetLutSnafu {
                                    action: "pushing data with ID and name",
                                })?;
                        }
                    }
                }
            }
            None => {
                if name.is_some() {
                    // Only a name
                    self.dataset
                        .lut
                        .append_name(name.unwrap(), entry)
                        .with_context(|_| DataSetLutSnafu {
                            action: "pushing data with name only",
                        })?;
                } else {
                    return Err(DataSetError::DataSetLut {
                        action: "pushing data",
                        source: LutError::NoKeyProvided,
                    });
                }
            }
        }

        buf.extend_from_slice(&this_buf);

        Ok(())
    }

    pub fn finalize(mut self, buf: Vec<u8>) -> Result<DataSet<T, ENTRIES>, DataSetError> {
        self.dataset.bytes = Bytes::copy_from_slice(&buf);
        self.dataset.set_crc32();
        Ok(self.dataset)
    }
}
