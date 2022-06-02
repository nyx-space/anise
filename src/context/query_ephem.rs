/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crate::{
    asn1::{context::AniseContext, time::Epoch},
    errors::{AniseError, InternalErrorKind},
    frame::Frame,
};
use der::Encode;
use log::warn;
use std::fs::File;
use std::io::Write;
use std::path::Path;

impl<'a> AniseContext<'a> {
    pub fn posvel_of_wrt(
        &self,
        of_frame: Frame,
        wrt_frame: Frame,
        epoch: Epoch,
    ) -> Result<[f64; 6], AniseError> {
        todo!()
    }

    /// Provided a state with its origin and orientation, returns that state with respect to the requested frame
    pub fn state_wrt(
        &self,
        orbit: [f64; 6],
        wrt_frame: Frame,
        epoch: Epoch,
    ) -> Result<[f64; 6], AniseError> {
        todo!()
    }
}
