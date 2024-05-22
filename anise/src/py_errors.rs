/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::almanac::metaload::MetaAlmanacError;
use crate::almanac::planetary::PlanetaryDataError;
use crate::ephemerides::EphemerisError;
use crate::errors::{AlmanacError, DecodingError, InputOutputError, IntegrityError, PhysicsError};
use crate::orientations::OrientationError;
use crate::structure::dataset::DataSetError;
use core::convert::From;

use pyo3::{exceptions::PyException, prelude::*};

impl From<PhysicsError> for PyErr {
    fn from(err: PhysicsError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl From<IntegrityError> for PyErr {
    fn from(err: IntegrityError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
impl From<DecodingError> for PyErr {
    fn from(err: DecodingError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
impl From<InputOutputError> for PyErr {
    fn from(err: InputOutputError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
impl From<AlmanacError> for PyErr {
    fn from(err: AlmanacError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
impl From<EphemerisError> for PyErr {
    fn from(err: EphemerisError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
impl From<OrientationError> for PyErr {
    fn from(err: OrientationError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl From<PlanetaryDataError> for PyErr {
    fn from(err: PlanetaryDataError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl From<MetaAlmanacError> for PyErr {
    fn from(err: MetaAlmanacError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
impl From<DataSetError> for PyErr {
    fn from(err: DataSetError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
