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
    astro::{Aberration, AzElRange},
    constants::SPEED_OF_LIGHT_KM_S,
    ephemerides::{EphemerisError, EphemerisPhysicsSnafu},
    errors::{AlmanacError, EphemerisSnafu, PhysicsError},
    frames::Frame,
    math::angles::{between_0_360, between_pm_180},
    prelude::Orbit,
    structure::{
        dataset::DataSetError, instrument::Instrument, location::Location, lookuptable::LutError,
    },
};

use super::Almanac;
use crate::errors::AlmanacResult;

use hifitime::TimeUnits;
use log::warn;

use snafu::ResultExt;

impl Almanac {
    /// Returns the Location from its ID, searching through all loaded location datasets in reverse order.
    pub fn instrument_from_id(&self, id: i32) -> AlmanacResult<Instrument> {
        // for data in self.location_data.values().rev() {
        //     if let Ok(datum) = data.get_by_id(id) {
        //         return Ok(datum);
        //     }
        // }

        // Err(AlmanacError::TLDataSet {
        //     action: "AER for location",
        //     source: DataSetError::DataSetLut {
        //         action: "seeking location by ID",
        //         source: LutError::UnknownId { id },
        //     },
        // })
        todo!()
    }

    /// Returns the Location from its name, searching through all loaded location datasets in reverse order.
    pub fn instrument_from_name(&self, name: &str) -> AlmanacResult<Instrument> {
        // for data in self.location_data.values().rev() {
        //     if let Ok(datum) = data.get_by_name(name) {
        //         return Ok(datum);
        //     }
        // }

        // Err(AlmanacError::TLDataSet {
        //     action: "AER for location",
        //     source: DataSetError::DataSetLut {
        //         action: "seeking location by name",
        //         source: LutError::UnknownName {
        //             name: name.to_string(),
        //         },
        //     },
        // })
        todo!()
    }
}
