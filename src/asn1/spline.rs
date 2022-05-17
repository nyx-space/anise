use bytemuck::{try_cast_ref, try_cast_slice};
use crc32fast::hash;
use der::{asn1::OctetStringRef, Decode, Encode, Length, Reader, Tag, Writer};

use crate::{naif::daf::Endianness, parse_bytes_as, prelude::AniseError, DBL_SIZE};

// use super::time::Epoch;

/// Maximum interpolation degree for splines. This is needed for encoding and decoding of Splines in ASN1 using the `der` library.
pub const MAX_INTERP_DEGREE: usize = 32;

/// Defines the two kinds of splines supports: equal time steps (fixed window) or unequal time steps (also called sliding window)
pub enum SplineKind {
    FixedWindow {
        window_duration_s: f64,
    },
    SlidingWindow {
        /// Sliding window ephemerides may only span 4 centuries to constraint stack size
        indexes: [TimeIndex; 4],
    },
}

impl<'a> Encode for SplineKind {
    fn encoded_len(&self) -> der::Result<der::Length> {
        match self {
            Self::FixedWindow { window_duration_s } => (*window_duration_s).encoded_len(),
            Self::SlidingWindow { indexes: _indexes } => {
                todo!()
            }
        }
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        match self {
            Self::FixedWindow { window_duration_s } => (*window_duration_s).encode(encoder),
            Self::SlidingWindow { indexes: _indexes } => {
                todo!()
            }
        }
    }
}

impl<'a> Decode<'a> for SplineKind {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // Check the header tag to decode this CHOICE
        if decoder.peek_tag()? == Tag::Real {
            Ok(Self::FixedWindow {
                window_duration_s: decoder.decode()?,
            })
        } else {
            decoder.sequence(|sdecoder| {
                let indexes = sdecoder.decode()?;
                Ok(Self::SlidingWindow { indexes })
            })
        }
    }
}

pub struct TimeIndex {
    pub century: i16,
    pub nanoseconds: u64,
}

impl<'a> Encode for TimeIndex {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.century.encoded_len()? + self.nanoseconds.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.century.encode(encoder)?;
        self.nanoseconds.encode(encoder)
    }
}

impl<'a> Decode<'a> for TimeIndex {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            century: decoder.decode()?,
            nanoseconds: decoder.decode()?,
        })
    }
}

// #[derive(Enumerated)]
// #[repr(u8)]
// pub enum TrunctationStrategy {
//     None = 0,
//     TruncateLow = 1,
//     TruncateHigh = 2,
// }

// WARNING: How do I specify the start and end epochs for variable sized windows where the duration in the window is needed to rebuild the state?
// Is that some kind of header? If so, what's its size? If it's a high precision epoch, it would be 80 bits, but more likely people will provide 64 bit floats.
// Also, I can't use an offset from the index because the splines are built separately from the index via multithreading, so that would be difficult to build (would need to mutate the spline prior to encoding)

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

pub struct Splines<'a> {
    pub kind: SplineKind,
    pub config: SplineCoeffCount,
    /// Store the CRC32 checksum of the stored data. This should be checked prior to interpreting the data in the spline.
    pub data_checksum: u32,
    // TODO: Figure out how to properly add the covariance info, it's a bit hard because of the diag size
    // pub cov_position_coeff_len: u8,
    // pub cov_velocity_coeff_len: u8,
    // pub cov_acceleration_coeff_len: u8,
    pub data: &'a [u8],
}

impl<'a> Splines<'a> {
    pub fn fetch(
        &self,
        spline_idx: usize,
        coeff_idx: usize,
        coeff: Coefficient,
    ) -> Result<f64, AniseError> {
        self.check_integrity()?;
        let mut offset = self.config.spline_offset(spline_idx);
        // Calculate the f64's offset in this spline
        offset += match coeff {
            Coefficient::X => 0,
            Coefficient::Y => (self.config.degree as usize) * DBL_SIZE,
            Coefficient::Z => (2 * self.config.degree as usize) * DBL_SIZE,
            _ => todo!(),
        };
        offset += coeff_idx * DBL_SIZE;
        // TODO: use try_cast_ref to avoid copying
        if offset + DBL_SIZE <= self.data.len() {
            let ptr = &self.data[offset..offset + DBL_SIZE];
            return Ok(parse_bytes_as!(f64, ptr, Endianness::Big));
        } else {
            Err(AniseError::IndexingError)
        }
    }

    pub fn check_integrity(&self) -> Result<(), AniseError> {
        // Ensure that the data is correctly decoded
        let computed_chksum = hash(self.data);
        if computed_chksum == self.data_checksum {
            Ok(())
        } else {
            Err(AniseError::IntegrityError)
        }
    }
}

impl<'a> Encode for Splines<'a> {
    fn encoded_len(&self) -> der::Result<Length> {
        self.kind.encoded_len()?
            + self.config.encoded_len()?
            + self.data_checksum.encoded_len()?
            + OctetStringRef::new(self.data).unwrap().encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.kind.encode(encoder)?;
        self.config.encode(encoder)?;
        self.data_checksum.encode(encoder)?;
        OctetStringRef::new(self.data).unwrap().encode(encoder)
    }
}

impl<'a> Decode<'a> for Splines<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let kind = decoder.decode()?;
        let config = decoder.decode()?;
        let data_checksum = decoder.decode()?;
        let data_bytes: OctetStringRef = decoder.decode()?;
        Ok(Self {
            kind,
            config,
            data_checksum,
            data: data_bytes.as_bytes(),
        })
    }
}

/*
/// Spline defines all of the coefficients to interpolate any of the values of this state.
/// If the array is empty, it means the data for that parameter is non existent (this does NOT mean it is zero).
#[derive(Default, Debug)]
pub struct Spline<'a> {
    /// Interpolation validity start and end epochs.
    /// NOTE: that if the spline was generated backward, the duration may be negative.
    /// Compute the usable duration of this spline as follows:
    /// duration_in_seconds = spline.usable_end_epoch - spline.usable_start_epoch
    /// NOTE: this spline is defined without referencing its index in the ephemeris. In practice,
    /// this allows it to be generated on a separate threads and subsequently added to the binary treee
    /// representing the unequal time step ephemeris.
    /// NOTE: to determine the polynomial degree, peak at the length of each coordinate.
    pub start_epoch: Epoch,
    /// End epoch is the usable end epoch of this spline
    pub end_epoch: Epoch,
    /// State information (km)
    // pub x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    pub x: &'a [u8],
    /// State information (km)
    pub y: &'a [u8],
    /// State information (km)
    pub z: &'a [u8],
    /*
    /// State information (km)
    pub vx: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// State information (km)
    pub vy: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// State information (km)
    pub vz: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2)
    pub cov_x_x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2)
    pub cov_y_x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2)
    pub cov_y_y: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2)
    pub cov_z_x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2)
    pub cov_z_y: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2)
    pub cov_z_z: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vx_x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vx_y: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vx_z: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s^2)
    pub cov_vx_vx: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vy_x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vy_y: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vy_z: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s^2)
    pub cov_vy_vx: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s^2)
    pub cov_vy_vy: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vz_x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vz_y: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s)
    pub cov_vz_z: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s^2)
    pub cov_vz_vx: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s^2)
    pub cov_vz_vy: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// Covariance information (km^2/s^2)
    pub cov_vz_vz: SequenceOf<f64, MAX_INTERP_DEGREE>,
     */
}

impl<'a> Encode for Spline<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        // XXX: How to handle variable length of the f64 data?
        // Maybe just store as big endian bytes and that's it?
        // Then, I'd need to figure out how to encode what data is present and what isn't.
        // That could be a bit field of 27 items, each representing whether a given field is set. They will be assumed to be the same size, but that's probably wrong.

        let mut x_data_ieee754_be = [0x0; 8 * MAX_INTERP_DEGREE];
        let mut x_size = 0;
        for x in self.x {
            for (i, byte) in x.to_be_bytes().iter().enumerate() {
                x_data_ieee754_be[i] = *byte;
                x_size += 1;
            }
        }

        self.start_epoch.encoded_len()?
            + self.end_epoch.encoded_len()?
            + OctetStringRef::new(&x_data_ieee754_be[..x_size])?.encoded_len()?
            + OctetStringRef::new(&x_data_ieee754_be[..x_size])?.encoded_len()?
            + OctetStringRef::new(&x_data_ieee754_be[..x_size])?.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        encoder.encode(&self.start_epoch)?;
        encoder.encode(&self.end_epoch)?;
        let mut x_data_ieee754_be = [0x0; 8 * MAX_INTERP_DEGREE];
        let mut size = 0;
        for x in self.x {
            for (i, byte) in x.to_be_bytes().iter().enumerate() {
                x_data_ieee754_be[i] = *byte;
                size += 1;
            }
        }
        encoder.encode(&OctetStringRef::new(&x_data_ieee754_be[..size])?)?;
        encoder.encode(&OctetStringRef::new(&x_data_ieee754_be[..size])?)?;
        encoder.encode(&OctetStringRef::new(&x_data_ieee754_be[..size])?)
    }
}

impl<'a> Decode<'a> for Spline<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let start_epoch = decoder.decode()?;
        let end_epoch = decoder.decode()?;
        let x_data_ieee754_be: OctetStringRef = decoder.decode()?;
        let y_data_ieee754_be: OctetStringRef = decoder.decode()?;
        let z_data_ieee754_be: OctetStringRef = decoder.decode()?;

        Ok(Self {
            start_epoch,
            end_epoch,
            x: x_data_ieee754_be.as_bytes(),
            y: y_data_ieee754_be.as_bytes(),
            z: z_data_ieee754_be.as_bytes(),
            ..Default::default()
        })
    }
}

 */
