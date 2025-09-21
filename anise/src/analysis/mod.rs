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
        scalars: &[(ScalarExpr, Option<&str>)],
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

                        let ab_corr = spec.ab_corr;

                        for (expr, alias) in scalars.iter() {
                            data.insert(
                                alias.or(Some(&expr.to_string())).unwrap().to_string(),
                                expr.evaluate(orbit, ab_corr, almanac),
                            );
                        }
                        (epoch, Ok(data))
                    }
                    Err(e) => (epoch, Err(e)),
                },
            )
            .collect()
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
    use crate::analysis::specs::{OrthogonalFrame, Plane};
    use crate::astro::{Aberration, Location, TerrainMask};
    use crate::constants::frames::{EME2000, IAU_EARTH_FRAME, MOON_J2000, SUN_J2000, VENUS_J2000};
    use crate::prelude::Almanac;
    use crate::structure::LocationDataSet;
    use hifitime::{Epoch, TimeSeries, Unit};
    use rstest::*;

    #[fixture]
    fn almanac() -> Almanac {
        use std::path::PathBuf;

        // Build the new location
        let dsn_madrid = Location {
            latitude_deg: 40.427_222,
            longitude_deg: 4.250_556,
            height_km: 0.834_939,
            frame: IAU_EARTH_FRAME.into(),
            // Create a fake elevation mask to check that functionality
            terrain_mask: vec![
                TerrainMask {
                    azimuth_deg: 0.0,
                    elevation_mask_deg: 0.0,
                },
                TerrainMask {
                    azimuth_deg: 130.0,
                    elevation_mask_deg: 8.0,
                },
                TerrainMask {
                    azimuth_deg: 140.0,
                    elevation_mask_deg: 0.0,
                },
            ],
            // Ignore terrain mask for the test
            terrain_mask_ignored: true,
        };

        // Build a dataset with this single location
        let mut loc_data = LocationDataSet::default();
        loc_data
            .push(dsn_madrid, Some(123), Some("DSN Madrid"))
            .unwrap();

        let manifest_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));

        let mut almanac = Almanac::new(
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
        .unwrap();

        almanac.location_data = loc_data;

        almanac
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
    fn test_analysis_gen_report(almanac: Almanac) {
        // Try to compute the SMA of the Earth with respect to the Sun.
        let target_frame = FrameSpec::Loaded(EME2000);
        let observer_frame = FrameSpec::Loaded(MOON_J2000);

        let state = StateSpec {
            target_frame: target_frame.clone(),
            observer_frame,
            ab_corr: Aberration::NONE,
        };

        // Build the orthogonal VNC frame of the Earth ... isn't useful per-se
        // just a proof of concept, ensuring we normalize these vectors.
        let vnc = OrthogonalFrame::XY {
            x: VectorExpr::Unit(Box::new(VectorExpr::Velocity(state.clone()))),
            y: VectorExpr::Unit(Box::new(VectorExpr::OrbitalMomentum(state.clone()))),
        };

        let sun_state = StateSpec {
            target_frame,
            observer_frame: FrameSpec::Loaded(SUN_J2000),
            ab_corr: Aberration::LT,
        };

        // Project the Earth->Sun vector onto the VNC frame
        let proj = VectorExpr::Project {
            v: Box::new(VectorExpr::Negate(Box::new(VectorExpr::Unit(Box::new(
                VectorExpr::Radius(sun_state),
            ))))),
            frame: Box::new(vnc),
            plane: Some(Plane::XY),
        };

        println!("{proj}");

        let scalars = [
            ScalarExpr::Element(OrbitalElement::SemiMajorAxis),
            ScalarExpr::Element(OrbitalElement::Eccentricity),
            ScalarExpr::Element(OrbitalElement::Rmag),
            ScalarExpr::BetaAngle,
            ScalarExpr::SolarEclipsePercentage {
                eclipsing_frame: VENUS_J2000,
            },
            ScalarExpr::Norm(VectorExpr::Radius(state.clone())),
            ScalarExpr::DotProduct {
                a: VectorExpr::EccentricityVector(state.clone()),
                b: VectorExpr::Fixed {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            ScalarExpr::VectorX(VectorExpr::EccentricityVector(state.clone())),
            ScalarExpr::VectorY(VectorExpr::EccentricityVector(state.clone())),
            ScalarExpr::VectorZ(VectorExpr::EccentricityVector(state.clone())),
            // Test orbital momentum magnitude
            ScalarExpr::Norm(VectorExpr::CrossProduct {
                a: Box::new(VectorExpr::Radius(state.clone())),
                b: Box::new(VectorExpr::Velocity(state.clone())),
            }),
            ScalarExpr::Element(OrbitalElement::Hmag),
            ScalarExpr::AngleBetween {
                a: VectorExpr::Radius(state.clone()),
                b: VectorExpr::Velocity(state.clone()),
            },
            ScalarExpr::AzimuthFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::ElevationFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::RangeFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::RangeRateFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::VectorX(proj.clone()),
            ScalarExpr::VectorY(proj.clone()),
            ScalarExpr::VectorZ(proj.clone()),
        ];

        let cnt = scalars.len();

        let mut scalars_with_aliases = scalars.map(|s| (s, None));
        // Set an alias for the last three.
        scalars_with_aliases[cnt - 3].1 = Some("proj VNC X");
        scalars_with_aliases[cnt - 2].1 = Some("proj VNC Y");
        scalars_with_aliases[cnt - 1].1 = Some("proj VNC Z");

        let data = almanac.generate_report(
            &scalars_with_aliases,
            state,
            TimeSeries::inclusive(
                Epoch::from_gregorian_utc_at_midnight(2025, 1, 1),
                Epoch::from_gregorian_utc_at_noon(2025, 1, 2),
                Unit::Day * 0.5,
            ),
        );

        assert_eq!(data.len(), 4);

        let last_row = data.values().last().unwrap().as_ref().unwrap();

        println!("{last_row:?}");
        assert_eq!(last_row.len(), scalars_with_aliases.len());

        // Test that we correctly computed the norm of the cross product
        assert_eq!(
            last_row["Hmag (km)"],
            last_row["|Radius(Earth J2000 -> Moon J2000) ⨯ Velocity(Earth J2000 -> Moon J2000)|"]
        );

        for (k, v) in last_row.iter() {
            if k.contains("proj") {
                // Check that we have correctly defined the projections onto an othogonal frame
                assert!(v.as_ref().unwrap().abs() <= 1.0);
            }
        }
    }
}
