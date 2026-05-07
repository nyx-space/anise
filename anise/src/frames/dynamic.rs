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

use crate::almanac::Almanac;
use crate::constants::orientations::{
    orientation_name_from_id, ECLIPJ2000, FRAME_BIAS_DEPSBI_ARCSEC, FRAME_BIAS_DPSIBI_ARCSEC,
    FRAME_BIAS_DRA0_ARCSEC, ICRS, J2000, J2000_TO_ECLIPJ2000_ANGLE_RAD,
};
use crate::constants::ARCSEC_TO_RAD;
use crate::hifitime::Epoch;
use crate::math::rotation::{r1, r1_dot, r2, r3, r3_dot, DCM};
use crate::naif::daf::datatypes::Type2ChebyshevSet;
use crate::naif::daf::{DAFError, DafDataType, NAIFDataSet, NAIFSummaryRecord};
use crate::orientations::{BPCSnafu, OrientationInterpolationSnafu};
use crate::orientations::{OrientationError, OrientationPhysicsSnafu};
use crate::prelude::Frame;
use nalgebra::{Matrix3, Vector3};

use core::fmt;

pub const DYNAMIC_FRAME_PREFIX: u8 = 0xA0;

/// Dynamic frames in ANISE are encoded as an integer in the orientation ID of the frame.
///
/// # Encoding format
///
/// The format is as follows:
/// ```text
///   0xA0 FF AA BB
/// ```
/// Where
///  - 0xA0: ANISE dynamic frame prefix
///  - FF: Frame family, either Earth True of Date, Earth Mean of Date, Earth True Equator True Equinox, non-Earth True of Date, non-Earth Mean of Date
///  - AA: Primary Model, e.g., precession model for Earth TOD/MOD frames
///  - BB: Secondary Model, e.g. nutation model identifier for Earth TOD frame
///
/// For generic body mean of date and true of dates, the source_id is contractually an i32 for compatibility with the rest of NAIF IDs.
/// **However**, due to encoding limitations, the ID is in fact limited to being strictly positive between 0 and 65535, i.e. it MUST be
/// representable on a u16. This guarantees the ability to define an MOD or TOD frame against any arbitrary NAIF ID of a celestial object,
/// but never against a spacecraft ID. **Importantly:** the encoding will fail silently.
///
/// # Examples
///
/// - Frame Family for Earth models is prefixed by `0xA0 Ez .. ..` (note the `E` in hex)
/// - Frame Family for other Bodies is prefixed by `0xA0 Bz .. ..` (note the `B` in hex)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DynamicFrame {
    EarthMeanOfDate {
        precession: EarthPrecessionModel,
    },
    EarthTrueOfDate {
        precession: EarthPrecessionModel,
        nutation: EarthNutationModel,
    },
    /// For the Earth TEME, the equation of equinox SOFA model is chosen from the nutation model.
    /// IAU1980 -> EQEQ94; IAU2000A -> EE00A; IAU2000B -> EE00B; IAU2006 -> EE06A
    EarthTrueEquatorMeanEquinox {
        precession: EarthPrecessionModel,
        nutation: EarthNutationModel,
    },
    BodyMeanOfDate {
        source_id: i32,
    },
    BodyTrueOfDate {
        source_id: i32,
    },
}

impl TryFrom<u32> for DynamicFrame {
    type Error = OrientationError;
    fn try_from(orientation_u32: u32) -> Result<Self, Self::Error> {
        let prefix = (orientation_u32 >> 24) as u8;
        if prefix == DYNAMIC_FRAME_PREFIX {
            let ff = ((orientation_u32 >> 16) & 0xFF) as u8;
            let aa = ((orientation_u32 >> 8) & 0xFF) as u8;
            let bb = (orientation_u32 & 0xFF) as u8;

            let source_id = (orientation_u32 & 0xFFFF) as i32;

            match ff {
                0xE0 => Ok(Self::EarthMeanOfDate {
                    precession: EarthPrecessionModel::try_from(aa)?,
                }),
                0xE1 => Ok(Self::EarthTrueOfDate {
                    precession: EarthPrecessionModel::try_from(aa)?,
                    nutation: EarthNutationModel::try_from(bb)?,
                }),
                0xE2 => Ok(Self::EarthTrueEquatorMeanEquinox {
                    precession: EarthPrecessionModel::try_from(aa)?,
                    nutation: EarthNutationModel::try_from(bb)?,
                }),
                0xB0 => Ok(Self::BodyMeanOfDate { source_id }),
                0xB1 => Ok(Self::BodyTrueOfDate { source_id }),
                _ => Err(OrientationError::NotDynamicFrame {
                    detail: format!("0x{ff:X} is not a valid dynamic frame family"),
                }),
            }
        } else {
            Err(OrientationError::NotDynamicFrame {
                detail: format!("{prefix:x} must be 0xA0 for a dynamic frame"),
            })
        }
    }
}

impl From<DynamicFrame> for i32 {
    fn from(frame: DynamicFrame) -> Self {
        let id_u32 = match frame {
            DynamicFrame::EarthMeanOfDate { precession } => {
                let aa: u8 = precession.into();
                pack_id_earth(0xE0, aa as u32, 00)
            }
            DynamicFrame::EarthTrueOfDate {
                precession,
                nutation,
            } => {
                let aa: u8 = precession.into();
                let bb: u8 = nutation.into();
                pack_id_earth(0xE1, aa as u32, bb as u32)
            }
            DynamicFrame::EarthTrueEquatorMeanEquinox {
                precession,
                nutation,
            } => {
                let aa: u8 = precession.into();
                let bb: u8 = nutation.into();
                pack_id_earth(0xE2, aa as u32, bb as u32)
            }
            DynamicFrame::BodyMeanOfDate { source_id } => pack_id_generic(0xB0, source_id as u32),
            DynamicFrame::BodyTrueOfDate { source_id } => pack_id_generic(0xB1, source_id as u32),
        };

        id_u32 as i32
    }
}

fn pack_id_earth(ff: u32, aa: u32, bb: u32) -> u32 {
    (DYNAMIC_FRAME_PREFIX as u32) << 24 | ff << 16 | aa << 8 | bb
}

fn pack_id_generic(ff: u32, source_id: u32) -> u32 {
    (DYNAMIC_FRAME_PREFIX as u32) << 24 | ff << 16 | source_id & 0xFFFF
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EarthPrecessionModel {
    IAU1976,
    IAU2000,
    IAU2006,
}

impl TryFrom<u8> for EarthPrecessionModel {
    type Error = OrientationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::IAU1976),
            1 => Ok(Self::IAU2000),
            // Skipping 2 so that the 2006 model TOD ID is 0xA0 FE 03 03
            3 => Ok(Self::IAU2006),
            _ => Err(OrientationError::NotDynamicFrame {
                detail: format!("{value} invalid precession module; use 0 for IAU1976, 1 for IAU2000, 3 for IAU2006"),
            }),
        }
    }
}

impl Into<u8> for EarthPrecessionModel {
    fn into(self) -> u8 {
        match self {
            Self::IAU1976 => 0,
            Self::IAU2000 => 1,
            Self::IAU2006 => 3,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EarthNutationModel {
    IAU1980,
    IAU2000A,
    IAU2000B,
    IAU2006,
}

impl TryFrom<u8> for EarthNutationModel {
    type Error = OrientationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::IAU1980),
            1 => Ok(Self::IAU2000A),
            2 => Ok(Self::IAU2000B),
            3 => Ok(Self::IAU2006),
            _ => Err(OrientationError::NotDynamicFrame {
                detail: format!("{value} invalid precession module; use 0 for IAU1976, 1 for IAU2000, 2 for IAU2006"),
            }),
        }
    }
}

impl Into<u8> for EarthNutationModel {
    fn into(self) -> u8 {
        match self {
            Self::IAU1980 => 0,
            Self::IAU2000A => 1,
            Self::IAU2000B => 2,
            Self::IAU2006 => 3,
        }
    }
}

impl fmt::Display for DynamicFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BodyMeanOfDate { source_id } => match orientation_name_from_id(*source_id) {
                Some(name) => write!(f, "{name} MOD"),
                None => write!(f, "{source_id} MOD"),
            },
            Self::BodyTrueOfDate { source_id } => match orientation_name_from_id(*source_id) {
                Some(name) => write!(f, "{name} TOD"),
                None => write!(f, "{source_id} TOD"),
            },
            Self::EarthMeanOfDate { precession } => {
                write!(f, "Earth MOD ({precession:?})")
            }
            Self::EarthTrueOfDate {
                precession,
                nutation,
            } => {
                if Into::<u8>::into(*precession) == Into::<u8>::into(*nutation)
                    || (*precession == EarthPrecessionModel::IAU2000
                        && *nutation == EarthNutationModel::IAU2000B)
                {
                    // Only print one of the nutation model name since it's more specific.
                    write!(f, "Earth TOD ({nutation:?})")
                } else {
                    write!(f, "Earth TOD ({precession:?}, {nutation:?})")
                }
            }
            Self::EarthTrueEquatorMeanEquinox {
                precession,
                nutation,
            } => {
                if Into::<u8>::into(*precession) == Into::<u8>::into(*nutation)
                    || (*precession == EarthPrecessionModel::IAU2000
                        && *nutation == EarthNutationModel::IAU2000B)
                {
                    // Only print one of the nutation model name since it's more specific.
                    write!(f, "Earth TEME ({nutation:?})")
                } else {
                    write!(f, "Earth TEME ({precession:?}, {nutation:?})")
                }
            }
        }
    }
}

#[cfg(test)]
mod ut_dynamic_frame {
    use super::{DynamicFrame, EarthNutationModel, EarthPrecessionModel};

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
