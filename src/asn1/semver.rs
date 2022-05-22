use der::{asn1::OctetStringRef, Decode, Encode, Error, ErrorKind, Length, Reader, Writer};

/// Semantic versioning is used throughout ANISE
/// It is encoded as a single octet string of 3 bytes of content (prependded by 1 one tag byte and 1 length byte)
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
