/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::Almanac;
use crate::errors::{AlmanacPhysicsSnafu, AlmanacResult};
use crate::{
    astro::{Aberration, AzElRange},
    constants::SPEED_OF_LIGHT_KM_S,
    ephemerides::{EphemerisError, EphemerisPhysicsSnafu},
    errors::{AlmanacError, EphemerisSnafu, PhysicsError},
    frames::Frame,
    math::{
        angles::{between_0_360, between_pm_180},
        rotation::Quaternion,
    },
    prelude::Orbit,
    structure::{
        dataset::DataSetError, instrument::Instrument, location::Location, lookuptable::LutError,
    },
};
use hifitime::Epoch;

use hifitime::TimeUnits;
use log::warn;

use snafu::ResultExt;

impl Almanac {
    /// Returns the Instrument from its ID, searching through all loaded instrument datasets in reverse order.
    pub fn instrument_from_id(&self, id: i32) -> AlmanacResult<Instrument> {
        for data in self.instrument_data.values().rev() {
            if let Ok(datum) = data.get_by_id(id) {
                return Ok(datum);
            }
        }

        Err(AlmanacError::TLDataSet {
            action: "instrument from ID",
            source: DataSetError::DataSetLut {
                action: "seeking location by ID",
                source: LutError::UnknownId { id },
            },
        })
    }

    /// Returns the Instrument Location from its name, searching through all loaded instrument datasets in reverse order.
    pub fn instrument_from_name(&self, name: &str) -> AlmanacResult<Instrument> {
        for data in self.instrument_data.values().rev() {
            if let Ok(datum) = data.get_by_name(name) {
                return Ok(datum);
            }
        }

        Err(AlmanacError::TLDataSet {
            action: "instrument from name",
            source: DataSetError::DataSetLut {
                action: "seeking location by name",
                source: LutError::UnknownName {
                    name: name.to_string(),
                },
            },
        })
    }

    pub fn instrument_field_of_view_margin(
        &self,
        instrument_id: i32,
        sc_attitude_to_body: Quaternion,
        sc_observer_frame: Frame,
        target_state: Orbit,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<f64> {
        let instrument = self.instrument_from_id(instrument_id)?;
        // Compute the spacecraft state in the target state's frame
        let sc_state = self.transform(
            target_state.frame,
            sc_observer_frame,
            target_state.epoch,
            ab_corr,
        )?;

        instrument
            .fov_margin_deg(sc_attitude_to_body, sc_state, target_state)
            .context(AlmanacPhysicsSnafu {
                action: "instrument FOV",
            })
    }
}
