use der::{
    asn1::{OctetStringRef, SequenceOf, Utf8StringRef},
    Decode, Encode, Error, ErrorKind, Length, Reader, Writer,
};

use hifitime::Epoch;

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
        let as_octet_string = OctetStringRef::new(&data).unwrap();
        as_octet_string.encoded_len()
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let data: [u8; 3] = [self.major, self.minor, self.patch];
        let as_octet_string = OctetStringRef::new(&data).unwrap();
        as_octet_string.encode(encoder)
    }
}

impl<'a> Decode<'a> for Semver {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let data: OctetStringRef = decoder.decode()?;
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
    }
}

#[derive(Clone, Debug)]
pub struct Metadata<'a> {
    /// The ANISE version number. Can be used for partial decoding to determine whether a file is compatible with a library.
    pub anise_version: Semver,
    /// Date time of the creation of this file.
    pub creation_date: Epoch,
    /// Originator of the file, either an organization, a person, a tool, or a combination thereof
    pub originator: &'a str,
    /// Unique resource identifier to the metadata of this file. This is for FAIR compliance.
    pub metadata_uri: &'a str,
}

impl Default for Metadata<'_> {
    fn default() -> Self {
        Self {
            anise_version: ANISE_VERSION,
            creation_date: Epoch::now().unwrap(),
            originator: Default::default(),
            metadata_uri: Default::default(),
        }
    }
}

impl<'a> Encode for Metadata<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let (centuries, nanoseconds) = self.creation_date.to_tai_parts();
        self.anise_version.encoded_len()?
            + centuries.encoded_len()?
            + nanoseconds.encoded_len()?
            + Utf8StringRef::new(self.originator)?.encoded_len()?
            + Utf8StringRef::new(self.metadata_uri)?.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let (centuries, nanoseconds) = self.creation_date.to_tai_parts();
        self.anise_version.encode(encoder)?;
        centuries.encode(encoder)?;
        nanoseconds.encode(encoder)?;
        Utf8StringRef::new(self.originator)?.encode(encoder)?;
        Utf8StringRef::new(self.metadata_uri)?.encode(encoder)
    }
}

impl<'a> Decode<'a> for Metadata<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let anise_version = decoder.decode()?;
        let centuries = decoder.decode()?;
        let nanoseconds = decoder.decode()?;
        let originator: Utf8StringRef<'a> = decoder.decode()?;
        let metadata_uri: Utf8StringRef<'a> = decoder.decode()?;
        Ok(Self {
            anise_version,
            originator: originator.as_str(),
            creation_date: Epoch::from_tai_parts(centuries, nanoseconds),
            metadata_uri: metadata_uri.as_str(),
        })
    }
}

// TODO: Determine if this shouldn't be a single SeqOf with a tuple or a LUT Entry of {Hash, Index}
/// A LookUpTable enables O(1) access to any ephemeris data
#[derive(Default)]
pub struct LookUpTable {
    // TODO: Add CRC32 and pack the hashes in BE
    /// Hashes of the general hashing algorithm
    pub hashes: SequenceOf<u32, 16>,
    /// Corresponding index for each hash, may only have 65_535 entries
    pub indexes: SequenceOf<u16, 16>,
}

impl<'a> Encode for LookUpTable {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.hashes.encoded_len()? + self.indexes.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.hashes.encode(encoder)?;
        self.indexes.encode(encoder)
    }
}

impl<'a> Decode<'a> for LookUpTable {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            hashes: decoder.decode()?,
            indexes: decoder.decode()?,
        })
    }
}

#[derive(Default)]
pub struct TrajectoryFile<'a> {
    pub metadata: Metadata<'a>,
    pub ephemeris_lut: LookUpTable,
    pub orientation_lut: LookUpTable,
    pub ephemeris_data: SequenceOf<Ephemeris<'a>, 512>,
    // TODO: Add orientation data
}

impl<'a> Encode for TrajectoryFile<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.metadata.encoded_len()?
            + self.ephemeris_lut.encoded_len()?
            + self.orientation_lut.encoded_len()?
            + self.ephemeris_data.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.metadata.encode(encoder)?;
        self.ephemeris_lut.encode(encoder)?;
        self.orientation_lut.encode(encoder)?;
        self.ephemeris_data.encode(encoder)
    }
}

impl<'a> Decode<'a> for TrajectoryFile<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            metadata: decoder.decode()?,
            ephemeris_lut: decoder.decode()?,
            orientation_lut: decoder.decode()?,
            ephemeris_data: decoder.decode()?,
        })
    }
}
