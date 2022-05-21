use der::{asn1::SequenceOf, Decode, Encode, Reader, Writer};

// TODO: Determine if this shouldn't be a single SeqOf with a tuple or a LUT Entry of {Hash, Index}
/// A LookUpTable enables O(1) access to any ephemeris data
#[derive(Default)]
pub struct LookUpTable {
    /// Hashes of the general hashing algorithm
    pub hashes: SequenceOf<u32, 512>,
    /// Corresponding index for each hash, may only have 65_535 entries
    pub indexes: SequenceOf<u16, 512>,
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
