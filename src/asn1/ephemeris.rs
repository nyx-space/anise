use der::{
    asn1::SequenceOf,
    asn1::{SetOf, Utf8String},
    Decode, Decoder, Encode, Length,
};

use super::{common::InterpolationKind, spline::Splines, time::Epoch};

pub struct Ephemeris<'a> {
    pub name: &'a str,
    pub ref_epoch: Epoch,
    pub backward: bool,
    pub parent_ephemeris_hash: u32,
    pub orientation_hash: u32,
    pub interpolation_kind: InterpolationKind,
    pub splines: Splines<'a>,
}

impl<'a> Encode for Ephemeris<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        Utf8String::new(self.name)?.encoded_len()?
            + self.ref_epoch.encoded_len()?
            + self.backward.encoded_len()?
            + self.parent_ephemeris_hash.encoded_len()?
            + self.orientation_hash.encoded_len()?
            + self.interpolation_kind.encoded_len()?
            + self.splines.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        Utf8String::new(self.name)?.encode(encoder)?;
        self.ref_epoch.encode(encoder)?;
        self.backward.encode(encoder)?;
        self.parent_ephemeris_hash.encode(encoder)?;
        self.orientation_hash.encode(encoder)?;
        self.interpolation_kind.encode(encoder)?;
        self.splines.encode(encoder)
    }
}

impl<'a> Decode<'a> for Ephemeris<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        let name: Utf8String = decoder.decode()?;

        Ok(Self {
            name: name.as_str(),
            ref_epoch: decoder.decode()?,
            backward: decoder.decode()?,
            parent_ephemeris_hash: decoder.decode()?,
            orientation_hash: decoder.decode()?,
            interpolation_kind: decoder.decode()?,
            splines: decoder.decode()?,
        })
    }
}
