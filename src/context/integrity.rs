/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::error;

use crate::{
    structure::context::AniseContext,
    errors::{AniseError, IntegrityErrorKind},
};

impl<'a> AniseContext<'a> {
    pub fn check_integrity(&self) -> Result<(), AniseError> {
        // Ensure that the lookup tables and arrays have the same number of items
        if self.ephemeris_lut.hashes.len() != self.ephemeris_lut.indexes.len()
            || self.ephemeris_lut.hashes.len() != self.ephemeris_data.len()
        {
            error!("[integrity] ephemeris lookup table lengths mistmatch");
            return Err(AniseError::IntegrityError(IntegrityErrorKind::LookupTable));
        }
        if self.orientation_lut.hashes.len() != self.orientation_lut.indexes.len()
            || self.orientation_lut.hashes.len() != self.orientation_data.len()
        {
            error!("[integrity] orientation lookup table lengths mistmatch");
            return Err(AniseError::IntegrityError(IntegrityErrorKind::LookupTable));
        }
        // Check ephemeris integrity
        for index in self.ephemeris_lut.indexes.iter() {
            // Check that we can access each item from the LUT
            let data = self.ephemeris_data.get(*index as usize).ok_or_else(|| {
                error!("[integrity] {} not in ephemeris data list", index);
                AniseError::IntegrityError(IntegrityErrorKind::DataMissing)
            })?;
            // Check that the data is valid
            data.splines.check_integrity()?;
            // Check that the hashed name is in the LUT and corresponds to this index
            let lut_idx = self.ephemeris_lut.index_for_key(data.name).map_err(|_| {
                error!("[integrity] {} not in ephemeris lookup table", data.name);
                AniseError::IntegrityError(IntegrityErrorKind::LookupTable)
            })?;
            if lut_idx != *index {
                error!(
                    "[integrity] expected LUT index of {} for item {}",
                    lut_idx, index
                );
                return Err(AniseError::IntegrityError(IntegrityErrorKind::LookupTable));
            }
        }
        // Check ephemeris integrity
        for index in self.orientation_lut.indexes.iter() {
            // Check that we can access each item from the LUT
            let data = self.orientation_data.get(*index as usize).ok_or_else(|| {
                error!("[integrity] {} not in orientation data list", index);
                AniseError::IntegrityError(IntegrityErrorKind::DataMissing)
            })?;
            // Check that the data is valid
            data.splines.check_integrity()?;
            // Check that the hashed name is in the LUT and corresponds to this index
            let lut_idx = self.orientation_lut.index_for_key(data.name).map_err(|_| {
                error!("[integrity] {} not in orientation lookup table", data.name);
                AniseError::IntegrityError(IntegrityErrorKind::LookupTable)
            })?;
            if lut_idx != *index {
                error!(
                    "[integrity] expected LUT index of {} for item {}",
                    lut_idx, index
                );
                return Err(AniseError::IntegrityError(IntegrityErrorKind::LookupTable));
            }
        }
        Ok(())
    }
}
