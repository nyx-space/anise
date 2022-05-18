use crc32fast::hash;
use der::{asn1::OctetStringRef, Decode, Encode, Length, Reader, Writer};

use super::splinekind::SplineKind;
use crate::{naif::daf::Endianness, parse_bytes_as, prelude::AniseError, DBL_SIZE};

#[derive(Debug, Default)]
pub struct SplineCoeffCount {
    pub degree: u8,
    pub num_epochs: u8,
    pub num_position_coeffs: u8,
    pub num_position_dt_coeffs: u8,
    pub num_velocity_coeffs: u8,
    pub num_velocity_dt_coeffs: u8,
}

impl SplineCoeffCount {
    /// Returns the offset (in bytes) in the octet string
    pub fn spline_offset(&self, idx: usize) -> usize {
        idx * self.len()
    }

    /// Returns the length of a spline in bytes
    pub fn len(&self) -> usize {
        let num_items: usize = (self.num_epochs
            + self.num_position_coeffs * self.degree
            + self.num_position_dt_coeffs * self.degree
            + self.num_velocity_coeffs * self.degree
            + self.num_velocity_dt_coeffs * self.degree)
            .into();
        DBL_SIZE * num_items
    }
}

impl Encode for SplineCoeffCount {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.degree.encoded_len()?
            + self.num_epochs.encoded_len()?
            + self.num_position_coeffs.encoded_len()?
            + self.num_position_dt_coeffs.encoded_len()?
            + self.num_velocity_coeffs.encoded_len()?
            + self.num_velocity_dt_coeffs.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.degree.encode(encoder)?;
        self.num_epochs.encode(encoder)?;
        self.num_position_coeffs.encode(encoder)?;
        self.num_position_dt_coeffs.encode(encoder)?;
        self.num_velocity_coeffs.encode(encoder)?;
        self.num_velocity_dt_coeffs.encode(encoder)
    }
}

impl<'a> Decode<'a> for SplineCoeffCount {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            degree: decoder.decode()?,
            num_epochs: decoder.decode()?,
            num_position_coeffs: decoder.decode()?,
            num_position_dt_coeffs: decoder.decode()?,
            num_velocity_coeffs: decoder.decode()?,
            num_velocity_dt_coeffs: decoder.decode()?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Coefficient {
    X,
    Xdt,
    Y,
    Ydt,
    Z,
    Zdt,
    VX,
    VXdt,
    VY,
    VYdt,
    VZ,
    VZdt,
}
