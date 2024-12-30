/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Writer};
use serde::{Deserialize, Serialize};

mod drag;
mod inertia;
mod mass;
mod srp;

use super::dataset::DataSetT;
pub use drag::DragData;
pub use inertia::Inertia;
pub use mass::Mass;
pub use srp::SRPData;

/// Spacecraft constants can store the some of the spacecraft constant data as the CCSDS Orbit Parameter Message (OPM) and CCSDS Attitude Parameter Messages (APM)
#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpacecraftData {
    /// Mass of the spacecraft in kg
    pub mass: Option<Mass>,
    /// Solar radiation pressure data
    pub srp_data: Option<SRPData>,
    /// Atmospheric drag data
    pub drag_data: Option<DragData>,
    // Inertia tensor
    pub inertia: Option<Inertia>,
}

impl DataSetT for SpacecraftData {
    const NAME: &'static str = "spacecraft data";
}

impl SpacecraftData {
    /// Specifies what data is available in this structure.
    ///
    /// Returns:
    /// + Bit 0 is set if `mass_kg` is available
    /// + Bit 1 is set if `srp_data` is available
    /// + Bit 2 is set if `drag_data` is available
    /// + Bit 3 is set if `inertia` is available
    fn available_data(&self) -> u8 {
        let mut bits: u8 = 0;

        if self.mass.is_some() {
            bits |= 1 << 0;
        }
        if self.srp_data.is_some() {
            bits |= 1 << 1;
        }
        if self.drag_data.is_some() {
            bits |= 1 << 2;
        }
        if self.inertia.is_some() {
            bits |= 1 << 3;
        }

        bits
    }
}

impl Encode for SpacecraftData {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let available_flags = self.available_data();
        available_flags.encoded_len()?
            + self.mass.encoded_len()?
            + self.srp_data.encoded_len()?
            + self.drag_data.encoded_len()?
            + self.inertia.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.available_data().encode(encoder)?;
        self.mass.encode(encoder)?;
        self.srp_data.encode(encoder)?;
        self.drag_data.encode(encoder)?;
        self.inertia.encode(encoder)
    }
}

impl<'a> Decode<'a> for SpacecraftData {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let data_flags: u8 = decoder.decode()?;

        let mass_kg = if data_flags & (1 << 0) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let srp_data = if data_flags & (1 << 1) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let drag_data = if data_flags & (1 << 2) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let inertia = if data_flags & (1 << 3) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        Ok(Self {
            mass: mass_kg,
            srp_data,
            drag_data,
            inertia,
        })
    }
}

#[cfg(test)]
mod spacecraft_constants_ut {
    use super::{Decode, DragData, Encode, Inertia, Mass, SRPData, SpacecraftData};

    #[test]
    fn sc_min_repr() {
        let repr = SpacecraftData::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn sc_with_srp_only() {
        let repr = SpacecraftData {
            srp_data: Some(SRPData::default()),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn sc_with_drag_only() {
        let repr = SpacecraftData {
            drag_data: Some(DragData::default()),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn sc_with_mass_only() {
        let repr = SpacecraftData {
            mass: Some(Mass::default()),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn sc_with_inertial_only() {
        let repr = SpacecraftData {
            inertia: Some(Inertia::default()),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn sc_with_srp_mass_inertia() {
        let repr = SpacecraftData {
            srp_data: Some(SRPData {
                area_m2: 2.0,
                coeff_reflectivity: 1.8,
            }),
            inertia: Some(Inertia {
                orientation_id: -20,
                i_xx_kgm2: 120.0,
                i_yy_kgm2: 180.0,
                i_zz_kgm2: 220.0,
                i_xy_kgm2: 20.0,
                i_xz_kgm2: -15.0,
                i_yz_kgm2: 30.0,
            }),
            mass: Some(Mass::from_dry_and_prop_masses(150.0, 50.6)),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn sc_full() {
        let repr = SpacecraftData {
            srp_data: Some(SRPData {
                area_m2: 2.0,
                coeff_reflectivity: 1.8,
            }),
            inertia: Some(Inertia {
                orientation_id: -20,
                i_xx_kgm2: 120.0,
                i_yy_kgm2: 180.0,
                i_zz_kgm2: 220.0,
                i_xy_kgm2: 20.0,
                i_xz_kgm2: -15.0,
                i_yz_kgm2: 30.0,
            }),
            mass: Some(Mass::from_dry_and_prop_masses(150.0, 50.6)),
            drag_data: Some(DragData::default()),
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
