use der::{
    asn1::{OctetString, SequenceOf, Utf8String},
    DateTime, Decode, Decoder, Encode, Error, ErrorKind, Length,
};

use super::ephemeris::Ephemeris;

pub const ANISE_VERSION: Semver = Semver {
    major: 0,
    minor: 0,
    patch: 1,
};

/// Semantic versioning is used throughout ANISE
/// It is encoded as a single octet string of 3 bytes of content (prependded by 1 one tag byte and 1 length byte)
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Semver {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl<'a> Encode for Semver {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let data: [u8; 3] = [self.major, self.minor, self.patch];
        let as_octet_string = OctetString::new(&data).unwrap();
        as_octet_string.encoded_len()
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        let data: [u8; 3] = [self.major, self.minor, self.patch];
        let as_octet_string = OctetString::new(&data).unwrap();
        as_octet_string.encode(encoder)
    }
}

impl<'a> Decode<'a> for Semver {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            let data: OctetString = decoder.decode()?;
            if data.len() != Length::new(3) {
                return Err(Error::new(
                    ErrorKind::Incomplete {
                        expected_len: Length::new(3),
                        actual_len: data.len(),
                    },
                    Length::new(0),
                ));
            }

            Ok(Self {
                major: data.as_bytes()[0],
                minor: data.as_bytes()[1],
                patch: data.as_bytes()[2],
            })
        })
    }
}

#[derive(Clone, Debug)]
pub struct Metadata<'a> {
    pub anise_version: Semver,
    pub file_version: Semver,
    pub originator: &'a str,
    pub creation_date: DateTime,
}

impl Default for Metadata<'_> {
    fn default() -> Self {
        Self {
            anise_version: ANISE_VERSION,
            file_version: Default::default(),
            originator: Default::default(),
            creation_date: DateTime::new(2022, 1, 1, 0, 0, 0).unwrap(),
        }
    }
}

impl<'a> Encode for Metadata<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.anise_version.encoded_len()?
            + self.file_version.encoded_len()?
            + Utf8String::new(self.originator)?.encoded_len()?
            + self.creation_date.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        self.anise_version.encode(encoder)?;
        self.file_version.encode(encoder)?;
        Utf8String::new(self.originator)?.encode(encoder)?;
        self.creation_date.encode(encoder)
    }
}

impl<'a> Decode<'a> for Metadata<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            let anise_version = decoder.decode()?;
            let file_version = decoder.decode()?;
            let originator: Utf8String<'a> = decoder.decode()?;
            let creation_date = decoder.decode()?;
            Ok(Self {
                anise_version,
                file_version,
                originator: originator.as_str(),
                creation_date,
            })
        })
    }
}

/// A LookUpTable enables O(1) access to any ephemeris data
#[derive(Default)]
pub struct LookUpTable {
    /// Hashes of the general hashing algorithm
    pub hashes: SequenceOf<u32, 16>,
    /// Corresponding index for each hash, may only have 65_535 entries
    pub indexes: SequenceOf<u16, 16>,
}

impl<'a> Encode for LookUpTable {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.hashes.encoded_len()? + self.indexes.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        self.hashes.encode(encoder)?;
        self.indexes.encode(encoder)
    }
}

impl<'a> Decode<'a> for LookUpTable {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            Ok(Self {
                hashes: decoder.decode()?,
                indexes: decoder.decode()?,
            })
        })
    }
}

#[derive(Default)]
pub struct TrajectoryFile<'a> {
    pub metadata: Metadata<'a>,
    pub ephemeris_lut: LookUpTable,
    pub orientation_lut: LookUpTable,
    pub ephemeris_data: SequenceOf<Ephemeris<'a>, 16>,
    // TODO: Add orientation data
}

impl<'a> Encode for TrajectoryFile<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.metadata.encoded_len()?
            + self.ephemeris_lut.encoded_len()?
            + self.orientation_lut.encoded_len()?
            + self.ephemeris_data.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        self.metadata.encode(encoder)?;
        self.ephemeris_lut.encode(encoder)?;
        self.orientation_lut.encode(encoder)?;
        self.ephemeris_data.encode(encoder)
    }
}

impl<'a> Decode<'a> for TrajectoryFile<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            Ok(Self {
                metadata: decoder.decode()?,
                ephemeris_lut: decoder.decode()?,
                orientation_lut: decoder.decode()?,
                ephemeris_data: decoder.decode()?,
            })
        })
    }
}
