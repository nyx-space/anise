/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crc32fast::hash;
use log::error;

use crate::{
    errors::{AniseError, IntegrityErrorKind, InternalErrorKind},
    naif::dafold::Endian,
    parse_bytes_as,
    structure::{
        common::InterpolationKind,
        spline::{Field, Splines},
    },
    DBL_SIZE,
};

impl<'a> Splines<'a> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of splines
    pub fn len(&self) -> usize {
        self.data.len() / self.metadata.len() + 1
    }

    pub fn fetch(
        &self,
        spline_idx: usize,
        coeff_idx: usize,
        field: Field,
    ) -> Result<f64, AniseError> {
        // Compute the index in bytes at which the data starts
        let offset = self.metadata.spline_offset(spline_idx)
            + self.metadata.field_offset(field, coeff_idx)?;

        // Safely fetch this coefficient, returning an error if we're out of bounds.
        match self.data.get(offset..offset + DBL_SIZE) {
            Some(ptr) => Ok(parse_bytes_as!(f64, ptr, Endian::Big)),
            None => {
                error!(
                    "[fetch] could not fetch {}-th {:?} in spline {}",
                    coeff_idx, field, spline_idx
                );
                Err(AniseError::MalformedData(offset + DBL_SIZE))
            }
        }
    }

    pub fn check_integrity(&self) -> Result<(), AniseError> {
        // Ensure that the data is correctly decoded
        let computed_chksum = hash(self.data);
        if computed_chksum == self.data_checksum {
            Ok(())
        } else {
            error!(
                "[integrity] expected hash {} but computed {}",
                self.data_checksum, computed_chksum
            );
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::ChecksumInvalid {
                    expected: self.data_checksum,
                    computed: computed_chksum,
                },
            ))
        }
    }

    /// Evaluate this spline at the requested epoch and returns the position only.
    pub fn position_at(
        &self,
        spline_idx: usize,
        offset_s: f64,
        window_length_s: f64,
        kind: InterpolationKind,
    ) -> Result<[f64; 3], AniseError> {
        let orbit = self.posvel_at(spline_idx, offset_s, window_length_s, kind)?;
        Ok([orbit[0], orbit[1], orbit[2]])
    }

    /// Evaluate this ephemeris at the requested epoch and returns the velocity only.
    pub fn velocity_at(
        &self,
        spline_idx: usize,
        offset_s: f64,
        window_length_s: f64,
        kind: InterpolationKind,
    ) -> Result<[f64; 3], AniseError> {
        let orbit = self.posvel_at(spline_idx, offset_s, window_length_s, kind)?;
        Ok([orbit[3], orbit[4], orbit[5]])
    }

    /// Evaluate this ephemeris at the requested epoch and returns an orbit structure.
    pub fn posvel_at(
        &self,
        _spline_idx: usize,
        _offset_s: f64,
        _window_length_s: f64,
        kind: InterpolationKind,
    ) -> Result<[f64; 6], AniseError> {
        if kind != InterpolationKind::ChebyshevSeries {
            return Err(InternalErrorKind::InterpolationNotSupported.into());
        }

        todo!()

        // let mut interp_t = [0.0_f64; MAX_DEGREE];
        // let mut interp_dt = [0.0_f64; MAX_DEGREE];

        // let t1 = 2.0 * offset_s / window_length_s - 1.0;
        // interp_t[0] = 1.0;
        // interp_t[1] = t1;
        // for i in 2..usize::from(self.config.degree) {
        //     interp_t[i] = (2.0 * t1) * interp_t[i - 1] - interp_t[i - 2];
        // }

        // interp_dt[0] = 0.0;
        // interp_dt[1] = 1.0;
        // interp_dt[2] = 2.0 * (2.0 * t1);
        // for i in 3..usize::from(self.config.degree) {
        //     interp_dt[i] = (2.0 * t1) * interp_dt[i - 1] - interp_dt[i - 2]
        //         + interp_t[i - 1]
        //         + interp_t[i - 1];
        // }
        // for interp_i in &mut interp_dt {
        //     *interp_i *= 2.0 / window_length_s;
        // }

        // let mut x = 0.0;
        // let mut y = 0.0;
        // let mut z = 0.0;
        // let mut vx = 0.0;
        // let mut vy = 0.0;
        // let mut vz = 0.0;

        // for (idx, pos_factor) in interp_t.iter().enumerate() {
        //     let vel_factor = interp_dt[idx];
        //     if self.config.num_position_coeffs > 0 {
        //         x += pos_factor * self.fetch(spline_idx, idx, Coefficient::X)?;
        //     }
        //     if self.config.num_position_coeffs > 1 {
        //         y += pos_factor * self.fetch(spline_idx, idx, Coefficient::Y)?;
        //     }
        //     if self.config.num_position_coeffs > 2 {
        //         z += pos_factor * self.fetch(spline_idx, idx, Coefficient::Z)?;
        //     }
        //     if self.config.num_velocity_coeffs > 0 {
        //         vx += vel_factor * self.fetch(spline_idx, idx, Coefficient::VX)?;
        //     }
        //     if self.config.num_velocity_coeffs > 1 {
        //         vy += vel_factor * self.fetch(spline_idx, idx, Coefficient::VY)?;
        //     }
        //     if self.config.num_velocity_coeffs > 2 {
        //         vz += vel_factor * self.fetch(spline_idx, idx, Coefficient::VZ)?;
        //     }
        // }

        // Ok([x, y, z, vx, vy, vz])
    }
}
