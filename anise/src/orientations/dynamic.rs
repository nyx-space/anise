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
use crate::constants::orientations::{ICRS, J2000};
use crate::frames::{DynamicFrame, EarthNutationModel, EarthPrecessionModel};
use crate::hifitime::{Epoch, TimeUnits, Unit};
use crate::math::Matrix3;
use crate::math::rotation::DCM;

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
            } => {
                // Evaluate nominal rotation matrix.
                //
                // P maps parent -> mean equator / mean equinox of date.
                // N maps mean-of-date -> true equator / true equinox of date.
                // Therefore TOD = N * P.
                let rot_mat = nutation.rot_mat(epoch) * precession.rot_mat(epoch);

                // Finite differencing for the time derivative.
                let pre_rot_dcm =
                    nutation.rot_mat(epoch - 1.seconds()) * precession.rot_mat(epoch - 1.seconds());

                let post_rot_dcm =
                    nutation.rot_mat(epoch + 1.seconds()) * precession.rot_mat(epoch + 1.seconds());

                let rot_mat_dt = (post_rot_dcm - pre_rot_dcm) * 0.5;

                let dcm = DCM {
                    rot_mat,
                    from: precession.parent_id(),
                    to: source.into(),
                    rot_mat_dt: Some(rot_mat_dt),
                };

                Ok(dcm)
            }
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
}

impl EarthPrecessionModel {
    /// Returns the parent ID, depends on the model used.
    pub(crate) fn parent_id(self) -> i32 {
        match self {
            EarthPrecessionModel::IAU1976 => J2000,
            EarthPrecessionModel::IAU2000 => ICRS,
            EarthPrecessionModel::IAU2006 => ICRS,
        }
    }

    /// Compute the rotation matrix using SOFA directly.
    pub(crate) fn rot_mat(&self, epoch: Epoch) -> Matrix3 {
        // SOFA models expect Terrestrial Time (TT) as a two-part Julian Date.
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

impl EarthNutationModel {
    pub(crate) fn rot_mat(&self, epoch: Epoch) -> Matrix3 {
        // SOFA models expect Terrestrial Time (TT) as a two-part Julian Date.
        let (tt1, tt2) = sofa_tt_jd_parts(epoch);

        let nmat = match self {
            EarthNutationModel::IAU1980 => sofars::pnp::nutm80(tt1, tt2),
            EarthNutationModel::IAU2000A => sofars::pnp::num00a(tt1, tt2),
            EarthNutationModel::IAU2000B => sofars::pnp::num00b(tt1, tt2),
            EarthNutationModel::IAU2006A => sofars::pnp::num06a(tt1, tt2),
        };

        // SOFARS returns a standard [[f64; 3]; 3] row-major array.
        // nalgebra's Matrix3::new populates row-by-row.
        Matrix3::new(
            nmat[0][0], nmat[0][1], nmat[0][2], nmat[1][0], nmat[1][1], nmat[1][2], nmat[2][0],
            nmat[2][1], nmat[2][2],
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
                nutation: EarthNutationModel::IAU2006A
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
                nutation: EarthNutationModel::IAU2006A
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
