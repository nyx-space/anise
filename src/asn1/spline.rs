use der::{asn1::SequenceOf, Decode, Decoder, Encode};

use super::time::Epoch;

/// Maximum interpolation degree for splines. This is needed for encoding and decoding of Splines in ASN1 using the `der` library.
pub const MAX_INTERP_DEGREE: usize = 32;

/// Spline defines all of the coefficients to interpolate any of the values of this state.
/// If the array is empty, it means the data for that parameter is non existent (this does NOT mean it is zero).
#[derive(Default, Debug)]
pub struct Spline {
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
    pub x: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// State information (km)
    pub y: SequenceOf<f64, MAX_INTERP_DEGREE>,
    /// State information (km)
    pub z: SequenceOf<f64, MAX_INTERP_DEGREE>,
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
}

impl<'a> Encode for Spline {
    fn encoded_len(&self) -> der::Result<der::Length> {
        // XXX: How to handle variable length of the f64 data?
        // Maybe just store as big endian bytes and that's it?
        // Then, I'd need to figure out how to encode what data is present and what isn't.
        // That could be a bit field of 27 items, each representing whether a given field is set. They will be assumed to be the same size, but that's probably wrong.

        self.start_epoch.encoded_len()?
            + self.end_epoch.encoded_len()?
            + self.x.encoded_len()?
            + self.y.encoded_len()?
            + self.z.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        encoder.encode(&self.start_epoch)?;
        encoder.encode(&self.end_epoch)?;
        encoder.encode(&self.x)?;
        encoder.encode(&self.y)?;
        encoder.encode(&self.z)
    }
}

impl<'a> Decode<'a> for Spline {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            Ok(Self {
                start_epoch: decoder.decode()?,
                end_epoch: decoder.decode()?,
                x: decoder.decode()?,
                y: decoder.decode()?,
                z: decoder.decode()?,
                ..Default::default()
            })
        })
    }
}
