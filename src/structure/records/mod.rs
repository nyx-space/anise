/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode};

use crate::prelude::AniseError;

pub trait Record<'a>: Encode + Decode<'a> {
    /// Returns whether or not the integrity of the data is correct.
    fn check_integrity(&self) -> Result<(), AniseError> {
        Ok(())
    }
}
