/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use core::fmt::Display;

/// Re-export hifitime's units as DurationUnit.
pub use hifitime::Unit as TimeUnit;

/// Defines the distance units supported by ANISE. This notably allows storing interpolation information from instruments to comets.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum LengthUnit {
    Micrometer,
    Millimeter,
    Meter,
    Kilometer,
    Megameter,
}

impl LengthUnit {
    /// Returns the conversion factor of this distance unit to meters.
    /// E.g. To convert Self::Kilometers into Self::Meters, multiply by 1e-3.
    #[must_use]
    pub const fn to_meters(&self) -> f64 {
        match self {
            Self::Micrometer => 1e6,
            Self::Millimeter => 1e3,
            Self::Meter => 1.0,
            Self::Kilometer => 1e-3,
            Self::Megameter => 1e-6,
        }
    }

    /// Returns the conversion factor of this distance unit from meters.
    /// E.g. To convert Self::Kilometers into Self::Meters, multiply by 1e3.
    #[must_use]
    pub const fn from_meters(&self) -> f64 {
        match self {
            Self::Micrometer => 1e-6,
            Self::Millimeter => 1e-3,
            Self::Meter => 1.0,
            Self::Kilometer => 1e3,
            Self::Megameter => 1e6,
        }
    }
}

impl Display for LengthUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Micrometer => write!(f, "um"),
            Self::Millimeter => write!(f, "mm"),
            Self::Meter => write!(f, "m"),
            Self::Kilometer => write!(f, "km"),
            Self::Megameter => write!(f, "Mm"),
        }
    }
}

impl Default for LengthUnit {
    fn default() -> Self {
        Self::Kilometer
    }
}
