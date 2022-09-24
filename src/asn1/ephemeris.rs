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
use hifitime::{asn1der::Asn1Epoch, Epoch, TimeSystem};

use crate::HashType;

use super::{common::InterpolationKind, spline::Splines};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ephemeris<'a> {
    pub name: &'a str,
    /// All epochs are encoded as high precision TDB durations since J2000 TDB.
    pub ref_epoch: Epoch,
    pub backward: bool,
    pub parent_ephemeris_hash: HashType,
    pub orientation_hash: HashType,
    pub interpolation_kind: InterpolationKind,
    pub splines: Splines<'a>,
}

impl<'a> Encode for Ephemeris<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        Utf8StringRef::new(self.name)?.encoded_len()?
            + self.ref_epoch.to_asn1(TimeSystem::TDB).encoded_len()?
            + self.backward.encoded_len()?
            + self.parent_ephemeris_hash.encoded_len()?
            + self.orientation_hash.encoded_len()?
            + self.interpolation_kind.encoded_len()?
            + self.splines.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        Utf8StringRef::new(self.name)?.encode(encoder)?;
        self.ref_epoch.to_asn1(TimeSystem::TDB).encode(encoder)?;
        self.backward.encode(encoder)?;
        self.parent_ephemeris_hash.encode(encoder)?;
        self.orientation_hash.encode(encoder)?;
        self.interpolation_kind.encode(encoder)?;
        self.splines.encode(encoder)
    }
}

impl<'a> Decode<'a> for Ephemeris<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let name: Utf8StringRef = decoder.decode()?;
        let ref_epoch_asn1: Asn1Epoch = decoder.decode()?;

        Ok(Self {
            name: name.as_str(),
            ref_epoch: ref_epoch_asn1.into(),
            backward: decoder.decode()?,
            parent_ephemeris_hash: decoder.decode()?,
            orientation_hash: decoder.decode()?,
            interpolation_kind: decoder.decode()?,
            splines: decoder.decode()?,
        })
    }
}
