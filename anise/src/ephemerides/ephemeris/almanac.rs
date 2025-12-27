/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::Ephemeris;
use crate::astro::Aberration;
use crate::prelude::{Almanac, Frame, Orbit};
use hifitime::TimeSeries;
use log::warn;
use rayon::prelude::*;
use std::collections::BTreeMap;

impl Almanac {
    /// Builds the ephemeris of the target seen from the observer with the provided aberration throughout the time series.
    pub fn build_ephemeris(
        &self,
        target_frame: Frame,
        observer_frame: Frame,
        time_series: TimeSeries,
        ab_corr: Option<Aberration>,
        object_id: String,
    ) -> Ephemeris {
        let states = time_series
            .par_bridge()
            .filter_map(|epoch| {
                self.transform(target_frame, observer_frame, epoch, ab_corr)
                    .map_or_else(
                        |e| {
                            warn!("{e}");
                            None
                        },
                        Some,
                    )
            })
            .collect::<Vec<Orbit>>();

        if states.is_empty() {
            warn!("empty ephemeris created");
        }

        let mut ephem = Ephemeris {
            object_id,
            interpolation: crate::naif::daf::DafDataType::Type13HermiteUnequalStep,
            degree: 7,
            state_data: BTreeMap::new(),
        };

        for state in states {
            ephem.insert_orbit(state);
        }

        ephem
    }
}
