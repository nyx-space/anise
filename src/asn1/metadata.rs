use der::{asn1::Utf8StringRef, Decode, Encode, Reader, Writer};

use hifitime::Epoch;

use super::{semver::Semver, ANISE_VERSION};

#[derive(Copy, Clone, Debug)]
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
