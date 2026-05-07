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
use crate::frames::{DynamicFrame, EarthNutationModel, EarthPrecessionModel};
use crate::hifitime::{Epoch, TimeUnits, Unit};
use crate::math::rotation::DCM;
use crate::math::Matrix3;
use crate::orientations::{BPCSnafu, OrientationInterpolationSnafu};

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
                // Evaluate nominal rotation matrix
                let rot_mat = precession.rot_mat(epoch);

                // Finite differencing for the time derivative
                let pre_rot_dcm = precession.rot_mat(epoch - 1.seconds());
                let post_rot_dcm = precession.rot_mat(epoch + 1.seconds());
                let rot_mat_dt = (post_rot_dcm - pre_rot_dcm) * 0.5;

                let dcm = DCM {
                    rot_mat,
                    from: precession.parent_id(),
                    to: source.into(),
                    rot_mat_dt: Some(rot_mat_dt),
                };

                Ok(dcm)
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

    // fn dynamic_rot_mat_dt<F>(
    //     &self,
    //     source: Frame,
    //     epoch: Epoch,
    //     f: F,
    // ) -> Result<Matrix3<f64>, OrientationError>
    // where
    //     F: Fn(&Almanac, Epoch) -> Result<Matrix3<f64>, OrientationError>,
    // {
    //     // Pick a small but not tiny step. 1 second is reasonable for precession/nutation
    //     // and body-pole frames. You can tune with CSPICE/SOFA regression tests.
    //     let dt = Unit::Second;

    //     let cp = f(self, epoch + dt)?;
    //     let cm = f(self, epoch - dt)?;

    //     Ok((cp - cm) * 0.5) // if dt is exactly 1 second
    // }
}

impl EarthPrecessionModel {
    pub(crate) fn rot_mat(&self, epoch: Epoch) -> Matrix3 {
        // SOFA models expect Terrestrial Time (TT) as a two-part Julian Date.
        // Assuming hifitime v4's `to_jde_tt_days()` or equivalent is available.
        let (tt1, tt2) = sofa_tt_jd_parts(epoch);

        let pmat = match self {
            EarthPrecessionModel::IAU1976 => sofars::pnp::pmat76(tt1, tt2),
            EarthPrecessionModel::IAU2000 => sofars::pnp::pmat00(tt1, tt2),
            EarthPrecessionModel::IAU2006 => sofars::pnp::pmat06(tt1, tt2),
        };

        // SOFARS returns a standard [[f64; 3]; 3] row-major array.
        // nalgebra's Matrix3::new populates row-by-row.
        Matrix3::new(
            pmat[0][0], pmat[0][1], pmat[0][2], pmat[1][0], pmat[1][1], pmat[1][2], pmat[2][0],
            pmat[2][1], pmat[2][2],
        )
    }
}

// Helper function to convert an epoch to a SOFA two-part epoch
fn sofa_tt_jd_parts(epoch: Epoch) -> (f64, f64) {
    let jde_tt = epoch.to_jde_tt_duration();

    let tt1 = jde_tt.to_unit(Unit::Day).trunc();
    let tt2 = (jde_tt - Unit::Day * tt1).to_unit(Unit::Day);

    (tt1, tt2)
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
