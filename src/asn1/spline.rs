use der::{asn1::OctetString, Decode, Decoder, Encode, Length, Tag};

// use super::time::Epoch;

/// Maximum interpolation degree for splines. This is needed for encoding and decoding of Splines in ASN1 using the `der` library.
pub const MAX_INTERP_DEGREE: usize = 32;

/// Defines the two kinds of splines supports: equal time steps (fixed window) or unequal time steps (also called sliding window)
pub enum SplineKind<'a> {
    FixedWindow {
        window_duration_s: f64,
    },
    SlidingWindow {
        /// Sliding window ephemerides may only span 4 centuries to constraint stack size
        indexes: [TimeIndex<'a>; 4],
    },
}

pub struct TimeIndex<'a> {
    pub century: i16,
    /// Nanoseconds are on 64 bit unsigned integer (u64) but stored as u8
    /// TODO: Figure out how to keep this as u64 (might have lifetime issues? Maybe easiest is to seek and convert as needed? But that's hard for a binary search)
    pub nanoseconds: &'a [u8],
}

impl<'a> Encode for TimeIndex<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.century.encoded_len()? + OctetString::new(self.nanoseconds).unwrap().encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        encoder.encode(&self.century)?;
        encoder.encode(&OctetString::new(self.nanoseconds).unwrap())
    }
}

impl<'a> Decode<'a> for TimeIndex<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        let century = decoder.decode()?;
        let ns_bytes: OctetString = decoder.decode()?;
        Ok(Self {
            century,
            nanoseconds: ns_bytes.as_bytes(),
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
    pub epochs: u8,
    pub position_coeffs: u8,
    pub position_dt_coeffs: u8,
    pub velocity_coeffs: u8,
    pub velocity_dt_coeffs: u8,
}

pub struct Splines<'a> {
    pub kind: SplineKind<'a>,
    pub config: SplineCoeffCount,
    // TODO: Add CRC32 for data integrity check before the transmute
    // TODO: Figure out how to properly add the covariance info, it's a bit hard because of the diag size
    // pub cov_position_coeff_len: u8,
    // pub cov_velocity_coeff_len: u8,
    // pub cov_acceleration_coeff_len: u8,
    pub data: &'a [u8],
}

impl<'a> Encode for SplineKind<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        match self {
            Self::FixedWindow { window_duration_s } => window_duration_s.encoded_len(),
            Self::SlidingWindow { indexes } => {
                let mut len = Length::new(2);
                for index in indexes {
                    len = (len + OctetString::new(index.nanoseconds).unwrap().encoded_len()?)?;
                }
                Ok(len)
            }
        }
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        match self {
            Self::FixedWindow { window_duration_s } => encoder.encode(window_duration_s),
            Self::SlidingWindow { indexes } => {
                encoder.sequence(Length::new(indexes.len() as u16), |sencoder| {
                    for index in indexes {
                        OctetString::new(index.nanoseconds)
                            .unwrap()
                            .encode(sencoder)?;
                    }
                    Ok(())
                })
            }
        }
    }
}

impl<'a> Decode<'a> for SplineKind<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
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

impl Encode for SplineCoeffCount {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.epochs.encoded_len()?
            + self.position_coeffs.encoded_len()?
            + self.position_dt_coeffs.encoded_len()?
            + self.velocity_coeffs.encoded_len()?
            + self.velocity_dt_coeffs.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        self.epochs.encode(encoder)?;
        self.position_coeffs.encode(encoder)?;
        self.position_dt_coeffs.encode(encoder)?;
        self.velocity_coeffs.encode(encoder)?;
        self.velocity_dt_coeffs.encode(encoder)
    }
}

impl<'a> Decode<'a> for SplineCoeffCount {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        Ok(Self {
            epochs: decoder.decode()?,
            position_coeffs: decoder.decode()?,
            position_dt_coeffs: decoder.decode()?,
            velocity_coeffs: decoder.decode()?,
            velocity_dt_coeffs: decoder.decode()?,
        })
    }
}

impl<'a> Encode for Splines<'a> {
    fn encoded_len(&self) -> der::Result<Length> {
        self.kind.encoded_len()?
            + self.config.encoded_len()?
            + OctetString::new(&self.data).unwrap().encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        self.kind.encode(encoder)?;
        self.config.encode(encoder)?;
        OctetString::new(&self.data).unwrap().encode(encoder)
    }
}

impl<'a> Decode<'a> for Splines<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        let kind = decoder.decode()?;
        let config = decoder.decode()?;
        let data_bytes: OctetString = decoder.decode()?;
        Ok(Self {
            kind,
            config,
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
            + OctetString::new(&x_data_ieee754_be[..x_size])?.encoded_len()?
            + OctetString::new(&x_data_ieee754_be[..x_size])?.encoded_len()?
            + OctetString::new(&x_data_ieee754_be[..x_size])?.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
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
        encoder.encode(&OctetString::new(&x_data_ieee754_be[..size])?)?;
        encoder.encode(&OctetString::new(&x_data_ieee754_be[..size])?)?;
        encoder.encode(&OctetString::new(&x_data_ieee754_be[..size])?)
    }
}

impl<'a> Decode<'a> for Spline<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        let start_epoch = decoder.decode()?;
        let end_epoch = decoder.decode()?;
        let x_data_ieee754_be: OctetString = decoder.decode()?;
        let y_data_ieee754_be: OctetString = decoder.decode()?;
        let z_data_ieee754_be: OctetString = decoder.decode()?;

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
