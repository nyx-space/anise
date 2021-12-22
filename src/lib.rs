/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate hifitime;

pub use hifitime::Epoch;

pub mod anise;
pub mod errors;
pub mod generated;

pub mod prelude {
    pub use crate::errors::AniseError;
    pub use crate::file_mmap;
    pub use crate::generated::anise_generated::anise::time::Epoch as AniseEpoch;
    pub use crate::generated::anise_generated::anise::{Anise, AniseArgs};
    pub use crate::generated::anise_generated::anise::{Metadata, MetadataArgs};
    pub use std::fs::File;
}

pub mod naif;
