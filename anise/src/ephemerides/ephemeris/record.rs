/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    analysis::prelude::OrbitalElement,
    astro::orbit_gradient::OrbitGrad,
    errors::{NoCovarianceSnafu, PhysicsError},
    prelude::Orbit,
};
use nalgebra::SMatrix;
use snafu::ensure;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub use super::{Covariance, LocalFrame};

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro", get_all))]
pub struct EphemerisRecord {
    /// Orbit of this ephemeris entry
    /// :rtype: Orbit
    pub orbit: Orbit,
    /// Optional covariance associated with this orbit
    /// :rtype: Covariance
    pub covar: Option<Covariance>,
}

#[cfg_attr(feature = "python", pymethods)]
impl EphemerisRecord {
    #[new]
    fn py_new(orbit: Orbit, covar: Option<Covariance>) -> Self {
        Self { orbit, covar }
    }

    /// Returns the covariance in the desired orbit local frame, or None if this record does not define a covariance.
    ///
    /// :type local_frame: LocalFrame
    /// :rtype: Covariance
    pub fn covar_in_frame(
        &self,
        local_frame: LocalFrame,
    ) -> Result<Option<Covariance>, PhysicsError> {
        match self.covar {
            None => Ok(None),
            Some(mut covar) => {
                // If it's already in the right frame, no-op
                if covar.local_frame == local_frame {
                    return Ok(Some(covar));
                }

                // Calculate Target Frame -> Inertial
                let desired_frame_to_inertial = self.orbit.dcm_to_inertial(local_frame)?;

                // Calculate Current Covar Frame -> Inertial
                let cur_frame_to_inertial = self.orbit.dcm_to_inertial(covar.local_frame)?;

                // M = R_target_to_inertial * (R_source_to_inertial)^T
                // M maps Source -> Target
                let dcm = (desired_frame_to_inertial.transpose() * cur_frame_to_inertial)?;

                // Apply 6x6 Rotation: P_new = M * P_old * M^T
                covar.matrix = dcm.state_dcm() * covar.matrix * dcm.state_dcm().transpose();
                covar.local_frame = local_frame;

                Ok(Some(covar))
            }
        }
    }

    /// Returns the 1-sigma uncertainty (Standard Deviation) for a given orbital element.
    ///
    /// The result is in the unit of the parameter (e.g., km for SMA, degrees for angles).
    ///
    /// This method uses the [OrbitGrad] structure (Hyperdual numbers) to compute the
    /// Jacobian of the element with respect to the inertial Cartesian state, and then
    /// rotates the covariance into that hyperdual dual space: J * P * J^T.
    ///
    /// :type oe: OrbitalElement
    /// :rtype: float
    pub fn sigma_for(&self, oe: OrbitalElement) -> Result<f64, PhysicsError> {
        ensure!(
            self.covar.is_some(),
            NoCovarianceSnafu {
                action: "compute orbital element uncertainty"
            }
        );

        // We know the covariance is defined now, let's make sure to grab its Inertial form.
        let covar = self.covar_in_frame(LocalFrame::Inertial)?.unwrap().matrix;

        // Build the rotation matrix using Orbit gradient.
        let orbit_dual = OrbitGrad::from(self.orbit);
        let xf_partial = orbit_dual.partial_for(oe)?;

        let rotmat = SMatrix::<f64, 1, 6>::new(
            xf_partial.wrt_x(),
            xf_partial.wrt_y(),
            xf_partial.wrt_z(),
            xf_partial.wrt_vx(),
            xf_partial.wrt_vy(),
            xf_partial.wrt_vz(),
        );

        Ok((rotmat * covar * rotmat.transpose())[(0, 0)].sqrt())
    }
}
