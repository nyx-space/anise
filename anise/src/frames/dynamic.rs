/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

#[cfg(feature = "python")]
use pyo3::exceptions::{PyTypeError, PyValueError};
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;
#[cfg(feature = "python")]
use pyo3::types::PyType;

use crate::constants::orientations::orientation_name_from_id;
use crate::orientations::OrientationError;

use core::fmt;

pub const DYNAMIC_FRAME_PREFIX: u8 = 0xA0;

/// Dynamic frames in ANISE are encoded as a packed integer within the frame's orientation ID.
///
/// # Encoding format
///
/// The identifier is a 32-bit signed integer packed as four bytes:
/// ```text
///   0xA0 FF AA BB
/// ```
/// Where:
///  - `0xA0`: ANISE dynamic frame prefix.
///  - `FF`: Frame family identifier (e.g., Earth Mean of Date, Body True of Date).
///  - `AA`: Primary payload (e.g., precession model for Earth frames, or the high byte of a source ID).
///  - `BB`: Secondary payload (e.g., nutation model for Earth frames, or the low byte of a source ID).
///
/// # Frame families
///
/// ```text
///   0xA0 E0 AA 00   Earth Mean Equator, Mean Equinox of Date (MOD)
///   0xA0 E1 AA BB   Earth True Equator, True Equinox of Date (TOD)
///   0xA0 E2 AA BB   Earth True Equator, Mean Equinox of Date (TEME)
///
///   0xA0 B0 SS SS   Body Mean of Date
///   0xA0 B1 SS SS   Body True of Date
/// ```
///
/// The `E*` families are Earth-specific models. The `B*` families are generic celestial-body pole models.
///
/// # Earth payload
///
/// For Earth frames, `AA` encodes the precession / bias-precession model:
///
/// ```text
///   0x00   IAU 1976 / FK5 precession
///   0x01   IAU 2000 precession-bias model
///   0x03   IAU 2006 precession-bias model
/// ```
///
/// *(Note: `0x02` is intentionally reserved and unused).*
///
/// For Earth TOD and TEME frames, `BB` encodes the nutation model:
///
/// ```text
///   0x00   IAU 1980 nutation
///   0x01   IAU 2000A nutation
///   0x02   IAU 2000B nutation
///   0x03   IAU 2006 / 2000A-compatible nutation
/// ```
///
/// For Earth MOD frames, `BB` is reserved and must strictly be `0x00`.
///
/// Earth MOD uses the selected precession model only. Earth TOD composes the selected precession and nutation models.
/// Earth TEME first builds the corresponding true-equator/true-equinox frame, then rotates about the true Z-axis by the equation of the equinoxes to replace the true equinox with the mean equinox.
///
/// For Earth TEME, the equation of the equinoxes model is strictly derived from the selected nutation model:
///
/// ```text
///   IAU1980  -> EQEQ94
///   IAU2000A -> EE00A
///   IAU2000B -> EE00B
///   IAU2006A  -> EE06A
/// ```
///
/// This aligns with the SOFA/SOFARS sidereal-time identity:
/// `apparent sidereal time = mean sidereal time + equation of the equinoxes`.
///
/// # Body payload
///
/// For generic body frames, `AA BB` is interpreted as a single unsigned 16-bit source orientation ID:
///
/// ```text
///   source_id = u16::from_be_bytes([AA, BB])
/// ```
///
/// While the public enum stores this as an `i32` for seamless integration with ANISE and NAIF ID routing, the compact bitmask fundamentally restricts the payload.
/// The source ID MUST be strictly positive and fall within the `0..=65535` range.
///
/// This perfectly accommodates standard celestial-body orientation IDs (e.g., `301` for the Moon, or `31001` for a lunar ME-style frame). **It cannot represent negative spacecraft IDs or deeply nested user-defined SPICE frames.**
/// **Warning:** Out-of-range body source IDs will fail silently via bitwise truncation. Callers must treat the `u16` bound as a strict mathematical contract.
///
/// Body TOD and MOD frames use the source orientation model solely to establish the body's pole direction. The source prime meridian (twist) angle is explicitly ignored.
///
/// - **Body True of Date** uses the full source pole model, inclusive of periodic trigonometric terms.
/// - **Body Mean of Date** uses the mean source pole model, zeroing out periodic trigonometric terms in the pole right ascension and declination.
///
/// For body TOD/MOD, the dynamic frame axes evaluate as follows via the same Euler rotations code at the PCK-defined IAU frames:
///
/// ```text
///   Z = source pole direction
///   X = normalize(parent_Z × Z)
///   Y = Z × X
/// ```
///
/// If `parent_Z × Z` evaluates as singular (i.e., the pole aligns with the inertial Z-axis), the fallback perfectly mirrors the Ansys STK specification:
///
/// ```text
///   Y = normalize(Z × parent_X)
///   X = Y × Z
/// ```
///
/// # Interaction with `Frame` fields
///
/// - `Frame::frozen_epoch`: If set, evaluates the dynamic models (precession, nutation, pole right ascension/declination) at the specified epoch rather than the integration time, freezing the frame inertially.
/// - `Frame::force_inertial`: If `true`, the time derivative of the resulting Direction Cosine Matrix (DCM) is explicitly zeroed out. The built-in Earth MOD/TOD constants are defined as inertial in the ANISE constants.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub enum DynamicFrame {
    /// Earth Mean Equator, Mean Equinox of Date.
    ///
    /// Also known as Earth Mean of Date or MEME.
    EarthMeanOfDate { precession: EarthPrecessionModel },
    /// Earth True Equator, True Equinox of Date.
    ///
    /// Also known as Earth True of Date or TETE.
    EarthTrueOfDate {
        precession: EarthPrecessionModel,
        nutation: EarthNutationModel,
    },
    /// The True Equator Mean Equinox (TEME) frame shares its True Equator (Z-axis) with the TOD frame,
    /// but backs out the nutation in right ascension to align the X-axis with the Mean Equinox.
    ///
    /// The Equation of the Equinoxes (EqE) SOFA model is determined by the `nutation` model:
    /// IAU1980 -> eqeq94; IAU2000A -> ee00a; IAU2000B -> ee00b; IAU2006A -> ee06a.
    ///
    /// The sign convention follows the sidereal-time identity used by SOFARS: apparent sidereal time
    /// is mean sidereal time plus the equation of the equinoxes.
    EarthTrueEquatorMeanEquinox {
        precession: EarthPrecessionModel,
        nutation: EarthNutationModel,
    },
    /// Generic body Mean of Date frame.
    ///
    /// The Z axis is the mean source pole. Periodic trigonometric terms in the
    /// source pole right ascension and declination are ignored. The source prime
    /// meridian angle is ignored.
    BodyMeanOfDate { source_id: i32 },
    /// Generic body True of Date frame.
    ///
    /// The Z axis is the full source pole, including periodic trigonometric
    /// terms. The source prime meridian angle is ignored.
    BodyTrueOfDate { source_id: i32 },
}

impl DynamicFrame {
    pub fn family(&self) -> &str {
        match self {
            Self::EarthMeanOfDate { .. } | Self::BodyMeanOfDate { .. } => "MOD",
            Self::EarthTrueOfDate { .. } | Self::BodyTrueOfDate { .. } => "TOD",
            Self::EarthTrueEquatorMeanEquinox { .. } => "TEME",
        }
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl DynamicFrame {
    #[allow(clippy::too_many_arguments)]
    #[classmethod]
    pub fn from_frame_id(_cls: &Bound<'_, PyType>, frame_id: i32) -> PyResult<Self> {
        Self::try_from(frame_id as u32).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn to_frame_id(&self) -> i32 {
        Into::<i32>::into(*self)
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self} (@{self:p})")
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }
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
                0xE0 => {
                    if bb != 0 {
                        return Err(OrientationError::NotDynamicFrame {
                            detail: format!("Earth MOD frame ID must end with 0x00, got 0x{bb:X}"),
                        });
                    }
                    Ok(Self::EarthMeanOfDate {
                        precession: EarthPrecessionModel::try_from(aa)?,
                    })
                }
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
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
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
            // Skipping 2 so that the 2006 model TOD ID is 0xA0 E1 03 03
            3 => Ok(Self::IAU2006),
            _ => Err(OrientationError::NotDynamicFrame {
                detail: format!(
                    "{value} invalid precession module; use 0 for IAU1976, 1 for IAU2000, 3 for IAU2006"
                ),
            }),
        }
    }
}

impl From<EarthPrecessionModel> for u8 {
    fn from(val: EarthPrecessionModel) -> Self {
        match val {
            EarthPrecessionModel::IAU1976 => 0,
            EarthPrecessionModel::IAU2000 => 1,
            EarthPrecessionModel::IAU2006 => 3,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub enum EarthNutationModel {
    IAU1980,
    IAU2000A,
    IAU2000B,
    IAU2006A,
}

impl TryFrom<u8> for EarthNutationModel {
    type Error = OrientationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::IAU1980),
            1 => Ok(Self::IAU2000A),
            2 => Ok(Self::IAU2000B),
            3 => Ok(Self::IAU2006A),
            _ => Err(OrientationError::NotDynamicFrame {
                detail: format!(
                    "{value} invalid nutation model; use 0 for IAU1980, 1 for IAU2000A, 2 for IAU2000B, 3 for IAU2006A"
                ),
            }),
        }
    }
}

impl From<EarthNutationModel> for u8 {
    fn from(val: EarthNutationModel) -> Self {
        match val {
            EarthNutationModel::IAU1980 => 0,
            EarthNutationModel::IAU2000A => 1,
            EarthNutationModel::IAU2000B => 2,
            EarthNutationModel::IAU2006A => 3,
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
                nutation: EarthNutationModel::IAU2006A
            }
        );
        let dynf_id: i32 = dynf.into();
        assert_eq!(dynf_id, -1595866365);
        assert_eq!(dynf_id as u32, 0xA0E1_0303);
        assert_eq!(format!("{dynf}"), "Earth TOD (IAU2006A)".to_string());

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
            "Earth TOD (IAU2000, IAU2006A)".to_string()
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
