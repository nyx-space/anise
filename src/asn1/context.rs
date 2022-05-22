use der::{asn1::SequenceOf, Decode, Encode, Reader, Writer};

use super::{ephemeris::Ephemeris, lookuptable::LookUpTable, metadata::Metadata};

#[derive(Default)]
pub struct AniseContext<'a> {
    pub metadata: Metadata<'a>,
    pub ephemeris_lut: LookUpTable,
    pub orientation_lut: LookUpTable,
    pub ephemeris_data: SequenceOf<Ephemeris<'a>, 256>,
    // TODO: Add orientation data
    pub orientation_data: SequenceOf<Ephemeris<'a>, 256>,
}

impl<'a> Encode for AniseContext<'a> {
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

impl<'a> Decode<'a> for AniseContext<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            metadata: decoder.decode()?,
            ephemeris_lut: decoder.decode()?,
            orientation_lut: decoder.decode()?,
            ephemeris_data: decoder.decode()?,
            ..Default::default()
        })
    }
}
