/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use snafu::ResultExt;

use super::{OrientationError, OrientationPhysicsSnafu};
use crate::almanac::Almanac;
use crate::constants::orientations::{
    orientation_name_from_id, ECLIPJ2000, FRAME_BIAS_DEPSBI_ARCSEC, FRAME_BIAS_DPSIBI_ARCSEC,
    FRAME_BIAS_DRA0_ARCSEC, ICRS, J2000, J2000_TO_ECLIPJ2000_ANGLE_RAD,
};
use crate::constants::ARCSEC_TO_RAD;
use crate::frames::{DynamicFrame, EarthNutationModel, EarthPrecessionModel};
use crate::hifitime::{Epoch, Unit};
use crate::math::rotation::{r1, r1_dot, r2, r3, r3_dot, DCM};
use crate::naif::daf::datatypes::Type2ChebyshevSet;
use crate::naif::daf::{DAFError, DafDataType, NAIFDataSet, NAIFSummaryRecord};
use crate::orientations::{BPCSnafu, OrientationInterpolationSnafu};
use crate::prelude::Frame;
use nalgebra::{Matrix3, Vector3};

use sofars::pnp::{
    nut00a, nut00b, nut06a, nut80, obl06, obl80, pmat00, pmat06, pmat76, pnm00a, pnm00b, pnm06a,
    pnm80,
};

use core::fmt;

impl Almanac {
    /// Builds the DCM to rotate from a dynamic frame.
    ///
    /// Notes:
    /// 1. If a fixed epoch is desired, it must be set BEFORE calling this function because the DynamicFrame does not freeze the epoch.
    /// 2. The DCM will include both the rotation matrix and its time derivative: if the frame should be inertial, the caller must handle that independently.
    pub fn rotation_to_parent_dynamic(
        &self,
        source: DynamicFrame,
        epoch: Epoch,
    ) -> Result<DCM, OrientationError> {
        match source {
            DynamicFrame::EarthMeanOfDate { precession } => {
                todo!("MOD")
            }
            DynamicFrame::EarthTrueOfDate {
                precession,
                nutation,
            } => todo!("TOD"),
            DynamicFrame::EarthTrueEquatorMeanEquinox {
                precession,
                nutation,
            } => todo!("TEME"),
            DynamicFrame::BodyMeanOfDate { source_id }
            | DynamicFrame::BodyTrueOfDate { source_id } => {
                let mean_model = matches!(source, DynamicFrame::BodyMeanOfDate { .. });

                for data in self.planetary_data.values().rev() {
                    if let Ok(planetary_data) = data.get_by_id(source_id) {
                        // Fetch the parent info
                        let system_data = match data.get_by_id(planetary_data.parent_id) {
                            Ok(parent) => parent,
                            Err(_) => planetary_data,
                        };

                        let mut dcm = planetary_data
                            .rotation_to_parent(epoch, &system_data, false, mean_model, true)
                            .context(OrientationPhysicsSnafu)?;
                        // Update the frame ID in the DCM
                        dcm.to = source.into();
                        return Ok(dcm);
                    }
                }

                Err(OrientationError::DynamicFrameNotLoaded {
                    source_id,
                    dyn_frame_id: Into::<i32>::into(source) as u32,
                })
            }
        }
    }

    // fn rotation_earth_mod(
    //     &self,
    //     source: Frame,
    //     epoch: Epoch,
    //     id: DynamicOrientationId,
    // ) -> Result<DCM, OrientationError> {
    //     let prec = SofaPrecessionModel::try_from(id.primary)?;
    //     let rot_mat = sofa_precession_matrix(prec, epoch)?;

    //     let rot_mat_dt = if source.force_inertial {
    //         None
    //     } else {
    //         Some(self.dynamic_rot_mat_dt(source, epoch, |almanac, t| {
    //             almanac.rotation_earth_mod_matrix(t, prec)
    //         })?)
    //     };

    //     Ok(DCM {
    //         rot_mat,
    //         rot_mat_dt,
    //         from: J2000,
    //         to: source.orientation_id,
    //     })
    // }

    // fn rotation_earth_tod(
    //     &self,
    //     source: Frame,
    //     epoch: Epoch,
    //     id: DynamicOrientationId,
    // ) -> Result<DCM, OrientationError> {
    //     let prec = SofaPrecessionModel::try_from(id.primary)?;
    //     let nut = SofaNutationModel::try_from(id.secondary)?;

    //     let rot_mat = sofa_true_of_date_matrix(prec, nut, epoch)?;

    //     let rot_mat_dt = if source.force_inertial {
    //         None
    //     } else {
    //         Some(self.dynamic_rot_mat_dt(source, epoch, |almanac, t| {
    //             almanac.rotation_earth_tod_matrix(t, prec, nut)
    //         })?)
    //     };

    //     Ok(DCM {
    //         rot_mat,
    //         rot_mat_dt,
    //         from: J2000,
    //         to: source.orientation_id,
    //     })
    // }

    // fn sofa_precession_matrix(
    //     model: SofaPrecessionModel,
    //     epoch: Epoch,
    // ) -> Result<Matrix3<f64>, OrientationError> {
    //     match model {
    //         SofaPrecessionModel::Iau1976 => {
    //             let (zeta, z, theta) = prec76(epoch)?;
    //             Ok(r3(-z) * r2(theta) * r3(-zeta))
    //         }
    //         SofaPrecessionModel::Iau2006 => {
    //             // Prefer SOFA pmat06-style matrix if exposed.
    //             pmat06(epoch)
    //         }
    //         SofaPrecessionModel::Iau2000 => pmat00(epoch),
    //         SofaPrecessionModel::None => todo!(),
    //     }
    // }

    // fn sofa_true_of_date_matrix(
    //     prec: SofaPrecessionModel,
    //     nut: SofaNutationModel,
    //     epoch: Epoch,
    // ) -> Result<Matrix3<f64>, OrientationError> {
    //     match (prec, nut) {
    //         (SofaPrecessionModel::Iau1976, SofaNutationModel::Iau1980) => {
    //             let p = sofa_precession_matrix(prec, epoch)?;
    //             let (dpsi, deps) = nut80(epoch)?;
    //             let eps_bar = obl80(epoch)?;
    //             let n = r1(-(eps_bar + deps)) * r3(-dpsi) * r1(eps_bar);
    //             Ok(n * p)
    //         }

    //         (SofaPrecessionModel::Iau2006, SofaNutationModel::Iau2000A) => {
    //             // Prefer SOFA pnm06a if exposed.
    //             pnm06a(epoch)
    //         }

    //         (SofaPrecessionModel::Iau2000, SofaNutationModel::Iau2000A) => pnm00a(epoch),

    //         (SofaPrecessionModel::Iau2000, SofaNutationModel::Iau2000B) => pnm00b(epoch),

    //         _ => todo!(),
    //     }
    // }

    // fn rotation_body_fixed_axis_date(
    //     &self,
    //     source: Frame,
    //     epoch: Epoch,
    //     id: DynamicOrientationId,
    //     _true_of_date: bool,
    // ) -> Result<DCM, OrientationError> {
    //     let fixed_source = FixedAxisSource::try_from(id.primary)?;

    //     let fixed_orientation_id =
    //         self.fixed_orientation_id_for_body(source.ephemeris_id, fixed_source)?;

    //     let fixed_frame = Frame {
    //         ephemeris_id: source.ephemeris_id,
    //         orientation_id: fixed_orientation_id,
    //         // Important: do not inherit the dynamic frame’s orientation_id.
    //         // Inherit force_inertial? No: we need the fixed orientation at epoch.
    //         force_inertial: false,
    //         frozen_epoch: None,
    //         ..source
    //     };

    //     let fixed_to_parent = self.rotation_to_parent(fixed_frame, epoch)?;
    //     let rot_mat = fixed_axis_date_from_fixed_dcm(&fixed_to_parent.rot_mat)?;

    //     let rot_mat_dt = if source.force_inertial {
    //         None
    //     } else {
    //         Some(self.dynamic_rot_mat_dt(source, epoch, |almanac, t| {
    //             let fixed_to_parent = almanac.rotation_to_parent(fixed_frame, t)?;
    //             fixed_axis_date_from_fixed_dcm(&fixed_to_parent.rot_mat)
    //         })?)
    //     };

    //     Ok(DCM {
    //         rot_mat,
    //         rot_mat_dt,
    //         from: J2000,
    //         to: source.orientation_id,
    //     })
    // }

    fn dynamic_rot_mat_dt<F>(
        &self,
        source: Frame,
        epoch: Epoch,
        f: F,
    ) -> Result<Matrix3<f64>, OrientationError>
    where
        F: Fn(&Almanac, Epoch) -> Result<Matrix3<f64>, OrientationError>,
    {
        // Pick a small but not tiny step. 1 second is reasonable for precession/nutation
        // and body-pole frames. You can tune with CSPICE/SOFA regression tests.
        let dt = Unit::Second;

        let cp = f(self, epoch + dt)?;
        let cm = f(self, epoch - dt)?;

        Ok((cp - cm) * 0.5) // if dt is exactly 1 second
    }
}

fn fixed_axis_date_from_fixed_dcm(
    fixed_from_parent: &Matrix3<f64>,
) -> Result<Matrix3<f64>, OrientationError> {
    let icrf_x = Vector3::new(1.0, 0.0, 0.0);
    let icrf_z = Vector3::new(0.0, 0.0, 1.0);

    // If C maps parent components to fixed components, rows are fixed axes in parent.
    let z = Vector3::new(
        fixed_from_parent[(2, 0)],
        fixed_from_parent[(2, 1)],
        fixed_from_parent[(2, 2)],
    )
    .normalize();

    let x_candidate = icrf_z.cross(&z);

    let (x, y) = if x_candidate.norm() > 1.0e-14 {
        let x = x_candidate.normalize();
        let y = z.cross(&x).normalize();
        (x, y)
    } else {
        let y = z.cross(&icrf_x).normalize();
        let x = y.cross(&z).normalize();
        (x, y)
    };

    Ok(Matrix3::new(
        x[0], x[1], x[2], y[0], y[1], y[2], z[0], z[1], z[2],
    ))
}

#[cfg(test)]
mod ut_dynamic_frame {
    use crate::orientations::dynamic::{DynamicFrame, EarthNutationModel, EarthPrecessionModel};

    #[test]
    fn encdec_earth() {
        let dynf = DynamicFrame::try_from(0xA0E1_0303).expect("should be valid");
        assert_eq!(
            dynf,
            DynamicFrame::EarthTrueOfDate {
                precession: EarthPrecessionModel::IAU2006,
                nutation: EarthNutationModel::IAU2006
            }
        );
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id, -1595866365);
        assert_eq!(dynf_id as u32, 0xA0E1_0303);
        assert_eq!(format!("{dynf}"), "Earth TOD (IAU2006)".to_string());

        let dynf = DynamicFrame::try_from(0xA0E0_0000).expect("should be valid");
        assert_eq!(
            dynf,
            DynamicFrame::EarthMeanOfDate {
                precession: EarthPrecessionModel::IAU1976,
            }
        );
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id as u32, 0xA0E0_0000);
        assert_eq!(format!("{dynf}"), "Earth MOD (IAU1976)".to_string());

        let dynf = DynamicFrame::try_from(0xA0E2_0101).expect("should be valid");
        assert_eq!(
            dynf,
            DynamicFrame::EarthTrueEquatorMeanEquinox {
                precession: EarthPrecessionModel::IAU2000,
                nutation: EarthNutationModel::IAU2000A
            }
        );
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id as u32, 0xA0E2_0101);
        assert_eq!(format!("{dynf}"), "Earth TEME (IAU2000A)".to_string());

        let dynf = DynamicFrame::try_from(0xA0E1_0102).expect("should be valid");
        assert_eq!(
            dynf,
            DynamicFrame::EarthTrueOfDate {
                precession: EarthPrecessionModel::IAU2000,
                nutation: EarthNutationModel::IAU2000B
            }
        );
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id as u32, 0xA0E1_0102);
        assert_eq!(format!("{dynf}"), "Earth TOD (IAU2000B)".to_string());

        let dynf = DynamicFrame::try_from(0xA0E1_0103).expect("should be valid");
        assert_eq!(
            dynf,
            DynamicFrame::EarthTrueOfDate {
                precession: EarthPrecessionModel::IAU2000,
                nutation: EarthNutationModel::IAU2006
            }
        );
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id as u32, 0xA0E1_0103);
        assert_eq!(
            format!("{dynf}"),
            "Earth TOD (IAU2000, IAU2006)".to_string()
        );
    }

    #[test]
    fn encdec_moon() {
        let dynf = DynamicFrame::try_from(0xA0B0_012D).expect("should be valid");
        assert_eq!(dynf, DynamicFrame::BodyMeanOfDate { source_id: 301 });
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id, -1599078099);
        assert_eq!(dynf_id as u32, 0xA0B0_012D);

        let dynf = DynamicFrame::try_from(0xA0B1_012D).expect("should be valid");
        assert_eq!(dynf, DynamicFrame::BodyTrueOfDate { source_id: 301 });
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id as u32, 0xA0B1_012D);
    }
}
