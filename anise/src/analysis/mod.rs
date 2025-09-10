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
    almanac::Almanac,
    analysis::expr::VectorExpr,
    errors::{AlmanacError, PhysicsError},
    prelude::Orbit,
};
use hifitime::{Epoch, TimeSeries};
use prelude::OrbitalElement;
use rayon::prelude::*;
use snafu::prelude::*;
use std::collections::HashMap;

pub mod elements;
pub mod expr;
pub mod framedef;
pub mod specs;

use expr::ScalarExpr;
use specs::StateSpec;

pub mod prelude {
    pub use super::elements::OrbitalElement;
    pub use super::expr::{ScalarExpr, VectorExpr};
    pub use super::specs::{FrameSpec, StateSpec};
    pub use crate::prelude::Frame;
}

// FOCI: 1. Build the angle between two objects, defined in the loaded Almanac.
//       2. Rebuild the angular momentum vector to demonstrate the cross product.

// TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed, enable Dhall serialization.
// Will be implemented in https://github.com/nyx-space/anise/issues/466
// use serde_derive::{Deserialize, Serialize};
// use serde_dhall::StaticType;

impl Almanac {
    pub fn generate_report(
        &self,
        scalars: &[ScalarExpr],
        state_spec: StateSpec,
        time_series: TimeSeries,
    ) -> HashMap<Epoch, Result<HashMap<String, AnalysisResult<f64>>, AnalysisError>> {
        time_series
            .par_bridge()
            .map_with(
                (&self, state_spec.clone(), scalars),
                |(almanac, spec, scalars), epoch| match spec.evaluate(epoch, almanac) {
                    Ok(orbit) => {
                        let mut data = HashMap::new();

                        for expr in scalars.iter() {
                            data.insert(expr.to_string(), expr.evaluate(orbit, almanac));
                        }
                        (epoch, Ok(data))
                    }
                    Err(e) => (epoch, Err(e)),
                },
            )
            .collect()
        //     .filter_map(|epoch| {

        //         match state_spec.evaluate(epoch, &self) {
        //             Ok(state) => {

        //             },
        //             Err(e) => e
        //         }

        //         self.transform(target_frame, observer_frame, epoch, ab_corr)
        //             .map_or_else(
        //                 |e| {
        //                     eprintln!("{e}");
        //                     None
        //                 },
        //                 Some,
        //             )
        //     })
        //     .collect::<Vec<CartesianState>>();
        // states.sort_by(|state_a, state_b| state_a.epoch.cmp(&state_b.epoch));
        // states;
        // todo!()
    }
}

#[derive(Debug, PartialEq, Snafu)]
#[snafu(visibility(pub))]
pub enum AnalysisError {
    #[snafu(display("computing {expr:?} on {state} encountered an Almanac error {source}"))]
    AlmanacExpr {
        expr: Box<ScalarExpr>,
        state: Box<Orbit>,
        #[snafu(source(from(AlmanacError, Box::new)))]
        source: Box<AlmanacError>,
    },
    #[snafu(display("computing state {spec:?} at {epoch} encountered an Almanac error {source}"))]
    AlmanacStateSpec {
        spec: Box<StateSpec>,
        epoch: Epoch,
        #[snafu(source(from(AlmanacError, Box::new)))]
        source: Box<AlmanacError>,
    },
    #[snafu(display("computing {el:?} on {orbit} encountered a physics error {source}"))]
    PhysicsOrbitEl {
        el: Box<OrbitalElement>,
        orbit: Box<Orbit>,
        #[snafu(source(from(PhysicsError, Box::new)))]
        source: Box<PhysicsError>,
    },
    #[snafu(display("computing {expr:?} at {epoch} encountered a physics error {source}"))]
    PhysicsVecExpr {
        expr: Box<VectorExpr>,
        epoch: Epoch,
        #[snafu(source(from(PhysicsError, Box::new)))]
        source: Box<PhysicsError>,
    },
}

pub type AnalysisResult<T> = Result<T, AnalysisError>;

#[cfg(test)]
mod ut_analysis {

    use crate::analysis::prelude::*;
    use crate::astro::Aberration;
    use crate::constants::frames::{EME2000, SUN_J2000};
    use crate::prelude::Almanac;
    use hifitime::{Epoch, TimeSeries, Unit};
    use rstest::*;

    #[fixture]
    fn almanac() -> Almanac {
        use std::path::PathBuf;

        let manifest_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));

        Almanac::new(
            &manifest_dir
                .clone()
                .join("../data/de440s.bsp")
                .to_string_lossy(),
        )
        .unwrap()
        .load(
            &manifest_dir
                .clone()
                .join("../data/pck08.pca")
                .to_string_lossy(),
        )
        .unwrap()
    }

    #[test]
    fn test_displays() {
        let from_frame = FrameSpec::Loaded(EME2000);
        let to_frame = FrameSpec::Loaded(SUN_J2000);

        let state = StateSpec {
            target_frame: from_frame,
            observer_frame: to_frame,
            ab_corr: Aberration::NONE,
        };

        assert_eq!(format!("{state}"), "Earth J2000 -> Sun J2000");

        let r = VectorExpr::Radius(state.clone());
        let v = VectorExpr::Velocity(state.clone());
        let h = VectorExpr::CrossProduct {
            a: Box::new(r.clone()),
            b: Box::new(v.clone()),
        };
        println!("{r}\n{v}\n{h}");
    }

    #[rstest]
    fn test_analysis_orbital_element(almanac: Almanac) {
        // Try to compute the SMA of the Earth with respect to the Sun.

        let target_frame = FrameSpec::Loaded(EME2000);
        let observer_frame = FrameSpec::Loaded(SUN_J2000);

        let state = StateSpec {
            target_frame,
            observer_frame,
            ab_corr: Aberration::NONE,
        };

        let scalars = [
            ScalarExpr::Element(OrbitalElement::SemiMajorAxis),
            ScalarExpr::Element(OrbitalElement::Eccentricity),
        ];

        let data = almanac.generate_report(
            &scalars,
            state,
            TimeSeries::inclusive(
                Epoch::from_gregorian_utc_at_midnight(2025, 1, 1),
                Epoch::from_gregorian_utc_at_noon(2025, 1, 2),
                Unit::Day * 0.5,
            ),
        );

        assert_eq!(data.len(), 4);

        println!("{data:?}");
    }
}
