/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::EARTH_J2000;
use anise::prelude::AniseError;
use anise::prelude::File;
use anise::Epoch;
use anise::{file_mmap, prelude::AniseContext};

/// Tests the ephemeris computations from the de438s which don't require any frame transformation.
#[test]
fn de438s_direct() {
    // TODO: Still need to define the API here.
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();
    let zero_vec = ctx
        .posvel_of_wrt(EARTH_J2000, EARTH_J2000, Epoch::now().unwrap())
        .unwrap();
    dbg!(zero_vec);
    // ctx.posvel_of_wrt_corr(Earth, Moon, LTCorr, epoch) -> position and velocity of the Earth with respect to the Moon with light time correction at epoch
    // ctx.quat()
}
