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
use crate::{
    errors::{AlmanacError, AlmanacResult},
    structure::{dataset::DataSetError, lookuptable::LutError, spacecraft::SpacecraftData},
    NaifId,
};

#[cfg(feature = "python")]
use pyo3::pymethods;

#[cfg_attr(feature = "python", pymethods)]
impl Almanac {
    /// Returns the SpacecraftData from its ID, searching through all loaded spacecraft datasets in reverse order.
    pub fn spacecraft_data_from_id(&self, id: NaifId) -> AlmanacResult<SpacecraftData> {
        for data in self.spacecraft_data.values().rev() {
            if let Ok(datum) = data.get_by_id(id) {
                return Ok(datum);
            }
        }

        Err(AlmanacError::TLDataSet {
            action: "spacecraft data from ID",
            source: DataSetError::DataSetLut {
                action: "seeking spacecraft data by ID",
                source: LutError::UnknownId { id },
            },
        })
    }

    /// Returns the SpacecraftData from its name, searching through all loaded spacecraft datasets in reverse order.
    pub fn spacecraft_data_from_name(&self, name: &str) -> AlmanacResult<SpacecraftData> {
        for data in self.spacecraft_data.values().rev() {
            if let Ok(datum) = data.get_by_name(name) {
                return Ok(datum);
            }
        }

        Err(AlmanacError::TLDataSet {
            action: "spacecraft data from name",
            source: DataSetError::DataSetLut {
                action: "seeking spacecraft data by name",
                source: LutError::UnknownName {
                    name: name.to_string(),
                },
            },
        })
    }
}

#[cfg(test)]
mod ut_spacecraft {
    use crate::prelude::Almanac;
    use crate::structure::spacecraft::{DragData, Inertia, Mass, SRPData, SpacecraftData};
    use crate::structure::SpacecraftDataSet;

    #[test]
    fn test_spacecraft_retrieval() {
        let sc_data1 = SpacecraftData {
            mass: Some(Mass::from_dry_and_prop_masses(100.0, 20.0)),
            srp_data: Some(SRPData {
                area_m2: 5.0,
                coeff_reflectivity: 1.2,
            }),
            drag_data: Some(DragData {
                area_m2: 2.0,
                coeff_drag: 2.1,
            }),
            inertia: Some(Inertia {
                orientation_id: -101,
                i_xx_kgm2: 10.0,
                i_yy_kgm2: 20.0,
                i_zz_kgm2: 30.0,
                i_xy_kgm2: 0.0,
                i_xz_kgm2: 0.0,
                i_yz_kgm2: 0.0,
            }),
        };

        let sc_data2 = SpacecraftData {
            mass: Some(Mass::from_dry_and_prop_masses(200.0, 50.0)),
            ..Default::default()
        };

        let mut sc_dataset = SpacecraftDataSet::default();
        sc_dataset.push(sc_data1, Some(-101), Some("SC1")).unwrap();
        sc_dataset.push(sc_data2, Some(-102), Some("SC2")).unwrap();

        let almanac = Almanac::default().with_spacecraft_data(sc_dataset);

        // Fetch by ID
        assert_eq!(almanac.spacecraft_data_from_id(-101).unwrap(), sc_data1);
        assert_eq!(almanac.spacecraft_data_from_id(-102).unwrap(), sc_data2);

        // Fetch by Name
        assert_eq!(almanac.spacecraft_data_from_name("SC1").unwrap(), sc_data1);
        assert_eq!(almanac.spacecraft_data_from_name("SC2").unwrap(), sc_data2);

        // Error cases
        assert!(almanac.spacecraft_data_from_id(-999).is_err());
        assert!(almanac.spacecraft_data_from_name("Unknown").is_err());
    }
}
