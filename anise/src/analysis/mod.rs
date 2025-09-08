/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

pub mod elements;
pub mod expr;
pub mod framedef;
pub mod specs;

use elements::OrbitalElement;
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

use hifitime::{Epoch, TimeSeries};
use std::{collections::HashMap, fmt};

use crate::{almanac::Almanac, astro::Aberration, errors::AlmanacError, prelude::Frame};
// TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed, enable Dhall serialization.
// Will be implemented in https://github.com/nyx-space/anise/issues/466
// use serde_derive::{Deserialize, Serialize};
// use serde_dhall::StaticType;

impl Almanac {
    pub fn generate_report(
        &self,
        scalars: &[ScalarExpr],
        state_def: StateSpec,
        timeseries: TimeSeries,
    ) -> Result<HashMap<Epoch, HashMap<String, f64>>, AlmanacError> {
        todo!()
    }
}

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
            from_frame,
            to_frame,
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

        let from_frame = FrameSpec::Loaded(EME2000);
        let to_frame = FrameSpec::Loaded(SUN_J2000);

        let state = StateSpec {
            from_frame,
            to_frame,
            ab_corr: Aberration::NONE,
        };

        let scalars = [
            ScalarExpr::Element(OrbitalElement::SemiMajorAxis),
            ScalarExpr::Element(OrbitalElement::Eccentricity),
        ];

        almanac
            .generate_report(
                &scalars,
                state,
                TimeSeries::inclusive(
                    Epoch::from_gregorian_tai_at_midnight(2025, 1, 1),
                    Epoch::from_gregorian_tai_at_noon(2026, 1, 1),
                    Unit::Minute * 1,
                ),
            )
            .unwrap();
    }
}
