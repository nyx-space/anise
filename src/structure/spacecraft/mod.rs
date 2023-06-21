/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{asn1::Utf8StringRef, Decode, Encode, Reader, Writer};

mod drag;
mod inertia;
mod mass;
mod srp;

pub use drag::DragData;
pub use inertia::Inertia;
pub use mass::Mass;
pub use srp::SRPData;

/// Spacecraft constants can store the same spacecraft constant data as the CCSDS Orbit Parameter Message (OPM) and CCSDS Attitude Parameter Messages (APM)
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct SpacecraftData<'a> {
    /// Name is used as the input for the hashing function
    pub name: &'a str,
    /// Generic comments field
    pub comments: &'a str,
    /// Mass of the spacecraft in kg
    pub mass_kg: Option<Mass>,
    /// Solar radiation pressure data
    pub srp_data: Option<SRPData>,
    /// Atmospheric drag data
    pub drag_data: Option<DragData>,
    // Inertia tensor
    pub inertia: Option<Inertia>,
}

impl<'a> SpacecraftData<'a> {
    /// Specifies what data is available in this structure.
    ///
    /// Returns:
    /// + Bit 0 is set if `mass_kg` is available
    /// + Bit 1 is set if `srp_data` is available
    /// + Bit 2 is set if `drag_data` is available
    /// + Bit 3 is set if `inertia` is available
    fn available_data(&self) -> u8 {
        let mut bits: u8 = 0;

        if self.mass_kg.is_some() {
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

impl<'a> Encode for SpacecraftData<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let available_flags = self.available_data();
        Utf8StringRef::new(self.name)?.encoded_len()?
            + Utf8StringRef::new(self.comments)?.encoded_len()?
            + available_flags.encoded_len()?
            + self.mass_kg.encoded_len()?
            + self.srp_data.encoded_len()?
            + self.drag_data.encoded_len()?
            + self.inertia.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        Utf8StringRef::new(self.name)?.encode(encoder)?;
        Utf8StringRef::new(self.comments)?.encode(encoder)?;
        self.available_data().encode(encoder)?;
        self.mass_kg.encode(encoder)?;
        self.srp_data.encode(encoder)?;
        self.drag_data.encode(encoder)?;
        self.inertia.encode(encoder)
    }
}

impl<'a> Decode<'a> for SpacecraftData<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let name: Utf8StringRef = decoder.decode()?;
        let comments: Utf8StringRef = decoder.decode()?;

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
            name: name.as_str(),
            comments: comments.as_str(),
            mass_kg,
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
        let repr = SpacecraftData {
            name: "demo spacecraft",
            comments: "this is an example of encoding spacecraft data",
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn sc_with_srp_only() {
        let repr = SpacecraftData {
            name: "demo spacecraft",
            comments: "this is an example of encoding spacecraft data",
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
            name: "demo spacecraft",
            comments: "this is an example of encoding spacecraft data",
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
            name: "demo spacecraft",
            comments: "this is an example of encoding spacecraft data",
            mass_kg: Some(Mass::default()),
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
            name: "demo spacecraft",
            comments: "this is an example of encoding spacecraft data",
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
            name: "demo spacecraft",
            comments: "this is an example of encoding spacecraft data",
            srp_data: Some(SRPData {
                area_m2: 2.0,
                coeff_reflectivity: 1.8,
            }),
            inertia: Some(Inertia {
                orientation_id: -20,
                i_11_kgm2: 120.0,
                i_22_kgm2: 180.0,
                i_33_kgm2: 220.0,
                i_12_kgm2: 20.0,
                i_13_kgm2: -15.0,
                i_23_kgm2: 30.0,
            }),
            mass_kg: Some(Mass::from_dry_and_fuel_masses(150.0, 50.6)),
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
            name: "demo spacecraft",
            comments: "this is an example of encoding spacecraft data",
            srp_data: Some(SRPData {
                area_m2: 2.0,
                coeff_reflectivity: 1.8,
            }),
            inertia: Some(Inertia {
                orientation_id: -20,
                i_11_kgm2: 120.0,
                i_22_kgm2: 180.0,
                i_33_kgm2: 220.0,
                i_12_kgm2: 20.0,
                i_13_kgm2: -15.0,
                i_23_kgm2: 30.0,
            }),
            mass_kg: Some(Mass::from_dry_and_fuel_masses(150.0, 50.6)),
            drag_data: Some(DragData::default()),
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SpacecraftData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
