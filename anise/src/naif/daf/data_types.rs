/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;

use snafu::ensure;

use crate::naif::daf::DatatypeSnafu;

use super::DAFError;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]

pub enum DataType {
    Type1ModifiedDifferenceArray = 1,
    Type2ChebyshevTriplet = 2,
    Type3ChebyshevSextuplet = 3,
    Type5DiscreteStates = 5,
    Type8LagrangeEqualStep = 8,
    Type9LagrangeUnequalStep = 9,
    Type10SpaceCommandTLE = 10,
    Type12HermiteEqualStep = 12,
    Type13HermiteUnequalStep = 13,
    Type14ChebyshevUnequalStep = 14,
    Type15PrecessingConics = 15,
    Type17Equinoctial = 17,
    Type18ESOCHermiteLagrange = 18,
    Type19ESOCPiecewise = 19,
    Type20ChebyshevDerivative = 20,
    Type21ExtendedModifiedDifferenceArray = 21,
}

impl TryFrom<i32> for DataType {
    type Error = DAFError;

    fn try_from(id: i32) -> Result<DataType, Self::Error> {
        ensure!(
            (1..=21).contains(&id),
            DatatypeSnafu {
                kind: "unknown data type",
                id
            }
        );
        match id {
            1 => Ok(DataType::Type1ModifiedDifferenceArray),
            2 => Ok(DataType::Type2ChebyshevTriplet),
            3 => Ok(DataType::Type3ChebyshevSextuplet),
            5 => Ok(DataType::Type5DiscreteStates),
            8 => Ok(DataType::Type8LagrangeEqualStep),
            9 => Ok(DataType::Type9LagrangeUnequalStep),
            10 => Ok(DataType::Type10SpaceCommandTLE),
            12 => Ok(DataType::Type12HermiteEqualStep),
            13 => Ok(DataType::Type13HermiteUnequalStep),
            14 => Ok(DataType::Type14ChebyshevUnequalStep),
            15 => Ok(DataType::Type15PrecessingConics),
            17 => Ok(DataType::Type17Equinoctial),
            18 => Ok(DataType::Type18ESOCHermiteLagrange),
            19 => Ok(DataType::Type19ESOCPiecewise),
            20 => Ok(DataType::Type20ChebyshevDerivative),
            21 => Ok(DataType::Type21ExtendedModifiedDifferenceArray),
            _ => Err(DAFError::Datatype {
                id,
                kind: "unknown data type",
            }),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DataType::Type1ModifiedDifferenceArray => "Modified differences",
                DataType::Type2ChebyshevTriplet => "Chebyshev Triplet",
                DataType::Type3ChebyshevSextuplet => "Chebyshev Sextuplet",
                DataType::Type5DiscreteStates => "Discrete States",
                DataType::Type8LagrangeEqualStep => "Lagrange EqualStep",
                DataType::Type9LagrangeUnequalStep => "Lagrange UnequalStep",
                DataType::Type10SpaceCommandTLE => "Space Command TLE",
                DataType::Type12HermiteEqualStep => "Hermite Equal Step",
                DataType::Type13HermiteUnequalStep => "Hermite Unequal Step",
                DataType::Type14ChebyshevUnequalStep => "Chebyshev UnequalStep",
                DataType::Type15PrecessingConics => "Precessing Conics",
                DataType::Type17Equinoctial => "Equinoctial",
                DataType::Type18ESOCHermiteLagrange => "ESOC Hermite Lagrange",
                DataType::Type19ESOCPiecewise => "ESOC Piecewise",
                DataType::Type20ChebyshevDerivative => "Chebyshev Derivative",
                DataType::Type21ExtendedModifiedDifferenceArray =>
                    "Extended Modified Difference Array",
            }
        )
    }
}

#[cfg(test)]
mod ut_datatype {
    use super::*;

    #[test]
    fn test_try_from_valid_values() {
        assert_eq!(
            DataType::try_from(1).unwrap(),
            DataType::Type1ModifiedDifferenceArray
        );
        assert_eq!(
            DataType::try_from(2).unwrap(),
            DataType::Type2ChebyshevTriplet
        );
        assert_eq!(
            DataType::try_from(3).unwrap(),
            DataType::Type3ChebyshevSextuplet
        );
        assert_eq!(
            DataType::try_from(5).unwrap(),
            DataType::Type5DiscreteStates
        );
        assert_eq!(
            DataType::try_from(8).unwrap(),
            DataType::Type8LagrangeEqualStep
        );
        assert_eq!(
            DataType::try_from(9).unwrap(),
            DataType::Type9LagrangeUnequalStep
        );
        assert_eq!(
            DataType::try_from(10).unwrap(),
            DataType::Type10SpaceCommandTLE
        );
        assert_eq!(
            DataType::try_from(12).unwrap(),
            DataType::Type12HermiteEqualStep
        );
        assert_eq!(
            DataType::try_from(13).unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert_eq!(
            DataType::try_from(14).unwrap(),
            DataType::Type14ChebyshevUnequalStep
        );
        assert_eq!(
            DataType::try_from(15).unwrap(),
            DataType::Type15PrecessingConics
        );
        assert_eq!(DataType::try_from(17).unwrap(), DataType::Type17Equinoctial);
        assert_eq!(
            DataType::try_from(18).unwrap(),
            DataType::Type18ESOCHermiteLagrange
        );
        assert_eq!(
            DataType::try_from(19).unwrap(),
            DataType::Type19ESOCPiecewise
        );
        assert_eq!(
            DataType::try_from(20).unwrap(),
            DataType::Type20ChebyshevDerivative
        );
        assert_eq!(
            DataType::try_from(21).unwrap(),
            DataType::Type21ExtendedModifiedDifferenceArray
        );
    }

    #[test]
    fn test_try_from_invalid_values() {
        let invalid_values = [0, 4, 6, 7, 11, 16, 22, 23, 100, -1, -5];
        for &value in &invalid_values {
            match DataType::try_from(value) {
                Ok(_) => panic!("Expected error for value {}", value),
                Err(e) => match e {
                    DAFError::Datatype { id, kind } => {
                        assert_eq!(id, value);
                        assert_eq!(kind, "unknown data type");
                    }
                    _ => panic!("Unexpected error variant"),
                },
            }
        }
    }

    #[test]
    fn trivial_display() {
        for data_type in [
            DataType::Type1ModifiedDifferenceArray,
            DataType::Type2ChebyshevTriplet,
            DataType::Type3ChebyshevSextuplet,
            DataType::Type5DiscreteStates,
            DataType::Type8LagrangeEqualStep,
            DataType::Type9LagrangeUnequalStep,
            DataType::Type10SpaceCommandTLE,
            DataType::Type12HermiteEqualStep,
            DataType::Type13HermiteUnequalStep,
            DataType::Type14ChebyshevUnequalStep,
            DataType::Type15PrecessingConics,
            DataType::Type17Equinoctial,
            DataType::Type18ESOCHermiteLagrange,
            DataType::Type19ESOCPiecewise,
            DataType::Type20ChebyshevDerivative,
            DataType::Type21ExtendedModifiedDifferenceArray,
        ] {
            let expected = match data_type {
                DataType::Type1ModifiedDifferenceArray => "Modified differences",
                DataType::Type2ChebyshevTriplet => "Chebyshev Triplet",
                DataType::Type3ChebyshevSextuplet => "Chebyshev Sextuplet",
                DataType::Type5DiscreteStates => "Discrete States",
                DataType::Type8LagrangeEqualStep => "Lagrange EqualStep",
                DataType::Type9LagrangeUnequalStep => "Lagrange UnequalStep",
                DataType::Type10SpaceCommandTLE => "Space Command TLE",
                DataType::Type12HermiteEqualStep => "Hermite Equal Step",
                DataType::Type13HermiteUnequalStep => "Hermite Unequal Step",
                DataType::Type14ChebyshevUnequalStep => "Chebyshev UnequalStep",
                DataType::Type15PrecessingConics => "Precessing Conics",
                DataType::Type17Equinoctial => "Equinoctial",
                DataType::Type18ESOCHermiteLagrange => "ESOC Hermite Lagrange",
                DataType::Type19ESOCPiecewise => "ESOC Piecewise",
                DataType::Type20ChebyshevDerivative => "Chebyshev Derivative",
                DataType::Type21ExtendedModifiedDifferenceArray => {
                    "Extended Modified Difference Array"
                }
            };

            assert_eq!(data_type.to_string(), expected);
        }
    }
}
