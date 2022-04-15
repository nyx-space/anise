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
    pub splines: Splines<'a>, // pub interpolator: Interpolator<'a>,
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

pub enum Interpolator<'a> {
    EqualTimeSteps(EqualTimeSteps<'a>),
    UnequalTimeSteps(UnequalTimeSteps<'a>),
}

impl<'a> Encode for Interpolator<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        match self {
            Self::EqualTimeSteps(ets) => ets.encoded_len(),
            Self::UnequalTimeSteps(uts) => uts.encoded_len(),
        }
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        match self {
            Self::EqualTimeSteps(ets) => ets.encode(encoder),
            Self::UnequalTimeSteps(uts) => uts.encode(encoder),
        }
    }
}

impl<'a> Decode<'a> for Interpolator<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        let maybe_ets: _ = decoder.decode();
        match maybe_ets {
            Ok(ets) => Ok(Self::EqualTimeSteps(ets)),
            Err(_) => Ok(Self::UnequalTimeSteps(decoder.decode()?)),
        }
    }
}

/// EqualTimeSteps defines an interpolated trajectory where each spline has the same duration, specified in seconds.
/// This method allows for O(1) access to any set of coefficients, thereby O(1) access to compute the position, and
/// optional velocity, of any ephemeris data.
/// This approach is commonly used to interpolate the position of slow moving objects like celestial objects.

#[derive(Default)]
pub struct EqualTimeSteps<'a> {
    /// The duration of this spline in seconds, used to compute the offset of the vectors to fetch.
    /// To retrieve the appropriate index in the coefficient data, apply the following algorithm.
    /// 0. Let `epoch` be the desired epoch, and `start_epoch` the start epoch of the parent structure.
    /// 1. Compute the offset between the desired epoch and the start_epoch:
    ///
    ///     `ephem_offset <- desired_epoch - start_epoch`
    /// 2. Compute the index in splines as:
    ///
    ///     `index <- floor( ephem_offset / spline_duration_s)`
    /// 3. Compute the time offset, in seconds, within that window:
    ///
    ///     `t_prime <- ephem_offset - index * spline_duration_s`
    /// 4. Retrieve the coefficient data at key `index`.
    /// 5. Initialize the proper interpolation scheme with `t_prime` as the requested interpolation time.
    pub spline_duration_s: f64,
    pub splines: &'a [Splines<'a>],
    // pub splines: SequenceOf<Spline<'a>, 50>,
}

impl<'a> Encode for EqualTimeSteps<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        // XXX: How to handle variable length of the f64 data?
        // Maybe just store as big endian bytes and that's it?
        // Then, I'd need to figure out how to encode what data is present and what isn't.
        // That could be a bit field of 27 items, each representing whether a given field is set. They will be assumed to be the same size, but that's probably wrong.

        self.spline_duration_s.encoded_len()?
            + self.splines.iter().fold(Length::new(2), |acc, spline| {
                (acc + spline.encoded_len().unwrap()).unwrap()
            })
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        encoder.encode(&self.spline_duration_s)?;
        encoder.sequence(Length::new(self.splines.len() as u16), |encoder| {
            for spline in self.splines {
                encoder.encode(spline)?;
            }
            Ok(())
        })
        // for spline in self.splines {
        //     encoder.encode(spline)?;
        // }
        // Ok(())
        // encoder.encode(&self.splines)
    }
}

impl<'a> Decode<'a> for EqualTimeSteps<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        let spline_duration_s = decoder.decode()?;
        let expected_len: u32 = decoder.peek_header().unwrap().length.into();
        dbg!(expected_len);
        // TODO: Consider switching back to SeqOf. It _may_ not be as problematic as the spline encoding since the latter uses octet string
        let mut splines: &'a [Splines<'a>; 1000];
        decoder.sequence(|decoder| {
            for idx in 0..expected_len {
                splines[idx as usize] = decoder.decode()?;
            }
            Ok(())
        });
        Ok(Self {
            spline_duration_s,
            splines,
        })
    }
}

/// UnequalTimeSteps defines an interpolated trajectory where each spline has its own duration.
/// This is common for spacecraft trajectories as the dynamics may vary greatly throughout the mission.
/// For example, an Earth orbiter's trajectory needs smaller splines to interpolate its Cartesian position over 1h30min
/// than a spacecraft in deep space between Jupiter and Saturn.
///
/// This structure provides a pre-sorted index of time offsets enabling an implementor to perform a binary search
/// for the desired coefficients. Hence, UnequalTimeSteps provides O(log(n)) access to any state data.
///
/// # Primer on building an interpolated trajectory
/// 1. Determine the interpolation scheme, which will determine the number of values (states) needed for the interpolation
/// 2. Determine the interpolation degree, which will determine the number of coefficients to calculate
/// 3. Assume N states are needed. Bucket N states and their associated epoch in a vector of size N.
/// 4. Set the start of the interpolation spline to the epoch of the first state.
/// 5. Normalize all state epochs between -1.0 and +1.0 (i.e. the first state's epoch is now -1.0 and the last is +1.0)
/// 6. Find the interpolation coefficients (ANISE will provide these algorithms).
/// 7. Optionally, verify that querying the interpolation at the initial epochs returns the original state data (x, y, z,... cov_vz_vz).
#[derive(Default)]
pub struct UnequalTimeSteps<'a> {
    /// A pre-sorted list of all of the offsets from the start_epoch of the Ephemeris
    /// available in the list of coefficient data.
    /// These time entries are centiseconds (10 milliseconds) past the start_epoch (defined
    /// in the parent Ephemeris object). Perform a binary search in this index to retrieve
    /// index for the desired epoch. Then, retrieve the Spline for that key.
    /// Ensure that the desired epoch is within the usable start and end time offsets.
    /// If the desired epoch is prior to that time, select the previous key and check again.
    /// If the desired epoch is after that time, select the next key and check again.
    /// If within usable time, call the appropriate interpolation function (using the parent's
    /// interpolation_kind attribute) with each of the coefficients as the polynominal weights.
    ///
    /// # Notes
    /// 1. The index is a signed integer of 64 bits because floating point values do not
    /// have the total order property, therefore we cannot guarantee an order thereby preventing
    /// a binary search of said vector. Further, it's as a signed integer to trivially support trajectories
    /// created with a forward and a backward propagation.
    /// 2. The index points to the start of the window. In theory, this should prevent the binary search
    /// from having to seek to the previous set of data compared to a method where the index points to the middle of the window.
    /// 3. We store the data in centiseconds because experience has shown that, in some high fidelity scenarios,
    /// a variable-duration spline may last less than one second (even for only 8 states). In practice, this leads to
    /// a collision in the indexing if it were to be in seconds. Therefore, a LIMITATION of this structure is that
    /// a variable-duration spline may only be up to 497 days long. If your trajectory is longer than that, you should
    /// convert it to an equal-time-step trajectory.
    // pub spline_time_index: &'a [i64],
    pub spline_time_index: SetOf<i64, 16>,
    // pub splines: &'a [Spline],
    pub splines: SequenceOf<Splines<'a>, 16>,
    pub time_normalization_min: f64,
    pub time_normalization_max: f64,
}

impl<'a> Encode for UnequalTimeSteps<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        // XXX: How to handle variable length of the f64 data?
        // Maybe just store as big endian bytes and that's it?
        // Then, I'd need to figure out how to encode what data is present and what isn't.
        // That could be a bit field of 27 items, each representing whether a given field is set. They will be assumed to be the same size, but that's probably wrong.

        self.spline_time_index.encoded_len()?
            + self.splines.encoded_len()?
            + self.time_normalization_min.encoded_len()?
            + self.time_normalization_max.encoded_len()?
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
        encoder.encode(&self.spline_time_index)?;
        encoder.encode(&self.splines)?;
        encoder.encode(&self.time_normalization_min)?;
        encoder.encode(&self.time_normalization_max)
    }
}

impl<'a> Decode<'a> for UnequalTimeSteps<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            Ok(Self {
                spline_time_index: decoder.decode()?,
                splines: decoder.decode()?,
                time_normalization_min: decoder.decode()?,
                time_normalization_max: decoder.decode()?,
            })
        })
    }
}
