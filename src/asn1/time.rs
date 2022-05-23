/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::der::{Decode, Encode, Reader, Writer};
use super::hifitime::{Epoch as EpochHifitime, TimeSystem};
use crate::prelude::AniseError;
use core::convert::From;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Epoch {
    pub epoch: EpochHifitime,
    pub system: TimeSystem,
}

impl From<EpochHifitime> for Epoch {
    fn from(epoch: EpochHifitime) -> Self {
        Self {
            epoch,
            system: TimeSystem::TAI,
        }
    }
}

impl Epoch {
    fn time_system_as_u8(&self) -> u8 {
        match self.system {
            TimeSystem::ET => 0,
            TimeSystem::TAI => 1,
            TimeSystem::TT => 2,
            TimeSystem::TDB => 3,
            TimeSystem::UTC => 4,
        }
    }

    fn time_system_from_u8(ts: u8) -> Result<TimeSystem, AniseError> {
        match ts {
            0 => Ok(TimeSystem::ET),
            1 => Ok(TimeSystem::TAI),
            2 => Ok(TimeSystem::TT),
            3 => Ok(TimeSystem::TDB),
            4 => Ok(TimeSystem::UTC),
            _ => Err(AniseError::InvalidTimeSystem),
        }
    }
}

impl Default for Epoch {
    fn default() -> Self {
        Self {
            epoch: EpochHifitime::from_gpst_nanoseconds(0),
            system: TimeSystem::TAI,
        }
    }
}

impl<'a> Encode for Epoch {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let (centuries, nanoseconds) = self.epoch.as_tai_duration().to_parts();
        centuries.encoded_len()?
            + nanoseconds.encoded_len()?
            + self.time_system_as_u8().encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let (centuries, nanoseconds) = self.epoch.as_tai_duration().to_parts();

        centuries.encode(encoder)?;
        nanoseconds.encode(encoder)?;
        self.time_system_as_u8().encode(encoder)
    }
}

impl<'a> Decode<'a> for Epoch {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let centuries = decoder.decode()?;
        let nanoseconds = decoder.decode()?;
        let ts_u8 = decoder.decode()?;
        let epoch = EpochHifitime::from_tai_parts(centuries, nanoseconds);
        let system = Epoch::time_system_from_u8(ts_u8).unwrap();
        Ok(Self { epoch, system })
    }
}
