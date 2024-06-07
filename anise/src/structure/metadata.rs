/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crate::errors::DecodingError;
use bytes::Bytes;
use core::fmt;
use core::str::FromStr;
use der::{asn1::Utf8StringRef, Decode, Encode, Reader, Writer};
use heapless::String;
use hifitime::Epoch;

/// Default maximum length of the Metadata originator length string
pub const MAX_ORIGINATOR_LEN: usize = 32;

use super::{dataset::DataSetType, semver::Semver, ANISE_VERSION};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// The ANISE version number. Can be used for partial decoding to determine whether a file is compatible with a library.
    pub anise_version: Semver,
    /// The type of dataset encoded in the rest of the structure
    pub dataset_type: DataSetType,
    /// Date time of the creation of this file.
    pub creation_date: Epoch,
    /// Originator of the file, either an organization, a person, a tool, or a combination thereof
    pub originator: String<MAX_ORIGINATOR_LEN>,
}

impl Metadata {
    /// Only decode the anise version and dataset type
    pub fn decode_header(bytes: &[u8]) -> Result<Self, DecodingError> {
        let anise_version =
            Semver::from_der(bytes.get(..5).ok_or(DecodingError::InaccessibleBytes {
                start: 0,
                end: 5,
                size: bytes.len(),
            })?)
            .map_err(|err| DecodingError::DecodingDer { err })?;
        let dataset_type = DataSetType::from_der(bytes.get(5..8).ok_or({
            DecodingError::InaccessibleBytes {
                start: 5,
                end: 8,
                size: bytes.len(),
            }
        })?)
        .map_err(|err| DecodingError::DecodingDer { err })?;
        let me = Self {
            anise_version,
            dataset_type,
            ..Default::default()
        };
        Ok(me)
    }

    pub fn from_bytes(buf: Bytes) -> Self {
        Self::from_der(&buf).unwrap()
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            anise_version: ANISE_VERSION,
            dataset_type: DataSetType::NotApplicable,
            creation_date: Epoch::now().unwrap(),
            originator: Default::default(),
        }
    }
}

impl Encode for Metadata {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.anise_version.encoded_len()?
            + self.dataset_type.encoded_len()?
            + Utf8StringRef::new(&format!("{}", self.creation_date))?.encoded_len()?
            + Utf8StringRef::new(&self.originator)?.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.anise_version.encode(encoder)?;
        self.dataset_type.encode(encoder)?;
        Utf8StringRef::new(&format!("{}", self.creation_date))?.encode(encoder)?;
        Utf8StringRef::new(&self.originator)?.encode(encoder)
    }
}

impl<'a> Decode<'a> for Metadata {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let anise_version = decoder.decode()?;
        let dataset_type = decoder.decode()?;
        let creation_date =
            Epoch::from_str(decoder.decode::<Utf8StringRef<'a>>()?.as_str()).unwrap();
        let orig_str = decoder.decode::<Utf8StringRef<'a>>()?.as_str();
        let originator = orig_str[..MAX_ORIGINATOR_LEN.min(orig_str.len())]
            .try_into()
            .unwrap();
        Ok(Self {
            anise_version,
            dataset_type,
            creation_date,
            originator,
        })
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ANISE version {}", self.anise_version)?;
        writeln!(
            f,
            "Originator: {}",
            if self.originator.is_empty() {
                "(not set)"
            } else {
                &self.originator
            }
        )?;
        writeln!(f, "Creation date: {}", self.creation_date)
    }
}

#[cfg(test)]
mod metadata_ut {

    use super::Metadata;
    use der::{Decode, Encode};

    #[test]
    fn meta_encdec_min_repr() {
        // A minimal representation of a planetary constant.
        let repr = Metadata::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Metadata::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        assert_eq!(
            format!("{repr}"),
            format!(
                r#"ANISE version ANISE version 0.4.0
Originator: (not set)
Creation date: {}
"#,
                repr_dec.creation_date
            )
        );
    }

    #[test]
    fn meta_invalid() {
        let repr = Metadata::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        // Check that we can decode the header only
        assert!(Metadata::decode_header(&buf).is_ok());
        // Check that reducing the number of bytes prevents decoding the header
        assert!(
            Metadata::decode_header(&buf[..7]).is_err(),
            "should not have enough for dataset"
        );
        assert!(
            Metadata::decode_header(&buf[..4]).is_err(),
            "should not have enough for version"
        );
    }

    #[test]
    fn meta_with_orig() {
        let repr = Metadata {
            originator: "Nyx Space Origin".try_into().unwrap(),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Metadata::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
