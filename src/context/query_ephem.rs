/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crate::{
    asn1::{context::AniseContext, ephemeris::Ephemeris, time::Epoch},
    constants::celestial_bodies::SUN,
    errors::{AniseError, IntegrityErrorKind, InternalErrorKind},
    frame::Frame,
};
use der::Encode;
use log::warn;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const MAX_PATH_LEN: usize = 5;

impl<'a> AniseContext<'a> {
    /// Try to return the ephemeris for the provided index, or returns an error.
    pub fn try_ephemeris_data(&self, idx: usize) -> Result<&'a Ephemeris, AniseError> {
        self.ephemeris_data
            .get(idx.into())
            .ok_or(AniseError::IntegrityError(IntegrityErrorKind::LookupTable))
    }

    /// Try to return the orientation for the provided index, or returns an error.
    pub fn try_orientation_data(&self, idx: usize) -> Result<&'a Ephemeris, AniseError> {
        self.orientation_data
            .get(idx.into())
            .ok_or(AniseError::IntegrityError(IntegrityErrorKind::LookupTable))
    }

    /// Try to construct the path from the source frame all the way to the solar system barycenter
    pub fn try_ephemeris_path(
        &self,
        source: &Frame,
    ) -> Result<(usize, [Option<u32>; MAX_PATH_LEN]), AniseError> {
        // Build a tree, set a fixed depth to avoid allocations
        // TODO: Consider switching to array vec or tinyvec
        let mut of_path = [None; MAX_PATH_LEN];
        let mut of_path_len = 0;
        let mut prev_ephem_hash = source.ephemeris_hash;
        for i in 0..MAX_PATH_LEN {
            // The solar system barycenter has a hash of 0.
            // TODO: Find a way to specify the true root of the context.
            let idx = self.ephemeris_lut.index_for_hash(&prev_ephem_hash)?;
            let parent_hash = self.try_ephemeris_data(idx.into())?.parent_ephemeris_hash;
            of_path[of_path_len] = Some(parent_hash);
            of_path_len += 1;
            if parent_hash == 0 {
                return Ok((of_path_len, of_path));
            }
        }
        Err(AniseError::MaxTreeDepth)
    }

    pub fn posvel_of_wrt(
        &self,
        of_frame: Frame,
        wrt_frame: Frame,
        epoch: Epoch,
    ) -> Result<[f64; 6], AniseError> {
        if of_frame == wrt_frame {
            // Both frames match
            return Ok([0.0; 6]);
        }
        // Grab the paths
        let (of_path_len, of_path) = self.try_ephemeris_path(&of_frame)?;
        let (wrt_path_len, wrt_path) = self.try_ephemeris_path(&wrt_frame)?;
        // Now that we have the paths, we can find the matching origin.

        todo!()
    }

    /// Provided a state with its origin and orientation, returns that state with respect to the requested frame
    pub fn state_wrt(
        &self,
        orbit: [f64; 6],
        wrt_frame: Frame,
        epoch: Epoch,
    ) -> Result<[f64; 6], AniseError> {
        todo!()
    }
}
