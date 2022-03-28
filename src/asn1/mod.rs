extern crate der;

// use der::{
//     asn1::{Any, OctetString},
//     Sequence,
// };

use der::{
    asn1::{BitString, OctetString},
    Decode, Decoder, DerOrd, Encode, Length, Sequence, ValueOrd,
};

// TODO:
// Replace the mid point and radius with start and end usable epoch

#[derive(Clone, Debug, PartialEq)]
pub struct SplineAsn1<'a> {
    pub rcrd_mid_point: f64,
    pub rcrd_radius_s: f64,
    pub x_data_ieee754: &'a OctetString<'a>,
    pub y_data_ieee754: &'a OctetString<'a>,
    pub z_data_ieee754: &'a OctetString<'a>,
    // pub x: Vec<f64>,
    // pub y: Vec<f64>,
    // pub z: Vec<f64>,
    // pub vx: &'a [f64],
    // pub vy: &'a [f64],
    // pub vz: &'a [f64],
    // pub cov_x_x: &'a [f64],
    // pub cov_y_x: &'a [f64],
    // pub cov_y_y: &'a [f64],
    // pub cov_z_x: &'a [f64],
    // pub cov_z_y: &'a [f64],
    // pub cov_z_z: &'a [f64],
    // pub cov_vx_x: &'a [f64],
    // pub cov_vx_y: &'a [f64],
    // pub cov_vx_z: &'a [f64],
    // pub cov_vx_vx: &'a [f64],
    // pub cov_vy_x: &'a [f64],
    // pub cov_vy_y: &'a [f64],
    // pub cov_vy_z: &'a [f64],
    // pub cov_vy_vx: &'a [f64],
    // pub cov_vy_vy: &'a [f64],
    // pub cov_vz_x: &'a [f64],
    // pub cov_vz_y: &'a [f64],
    // pub cov_vz_z: &'a [f64],
    // pub cov_vz_vx: &'a [f64],
    // pub cov_vz_vy: &'a [f64],
    // pub cov_vz_vz: &'a [f64],
}

impl<'a> SplineAsn1<'a> {}

impl<'a> Encode for SplineAsn1<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        // XXX: How to handle variable length of the f64 data?
        // Maybe just store as big endian bytes and that's it?
        // Then, I'd need to figure out how to encode what data is present and what isn't.
        // That could be a bit field of 27 items, each representing whether a given field is set. They will be assumed to be the same size, but that's probably wrong.

        self.rcrd_mid_point.encoded_len()?
            + self.rcrd_radius_s.encoded_len()?
            + self.x_data_ieee754.encoded_len()?
            + self.y_data_ieee754.encoded_len()?
            + self.z_data_ieee754.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        encoder.encode(&self.rcrd_mid_point)?;
        encoder.encode(&self.rcrd_radius_s)?;
        encoder.encode(self.x_data_ieee754)?;
        encoder.encode(self.y_data_ieee754)?;
        encoder.encode(self.z_data_ieee754)
    }
}

// impl<'a> Decode<'a> for SplineAsn1<'a> {
//     fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
//         decoder.sequence(|decoder| {
//             let usable_start_epoch = decoder.decode()?;
//             let usable_end_epoch = decoder.decode()?;

//             let x_data_ieee754: &'a OctetString = decoder.decode()?;

//             Ok(Self {
//                 usable_start_epoch,
//                 usable_end_epoch,
//                 x_data_ieee754,
//                 y_data_ieee754: &OctetString::new(&[0; 0]).unwrap(),
//                 z_data_ieee754: &OctetString::new(&[0; 0]).unwrap(),
//             })
//         })
//     }

//     fn from_der(bytes: &'a [u8]) -> der::Result<Self> {
//         let mut decoder = Decoder::new(bytes)?;
//         let result = Self::decode(&mut decoder)?;
//         decoder.finish(result)
//     }
// }
