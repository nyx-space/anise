/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::EARTH_J2000;
use anise::constants::orientations::J2000;
use anise::frame::Frame;
use anise::prelude::AniseError;
use anise::prelude::File;
use anise::Epoch;
use anise::{file_mmap, prelude::AniseContext};

/// Tests the ephemeris computations from the de438s which don't require any frame transformation.
#[test]
fn de438s_paths() {
    // TODO: Still need to define the API here.
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();

    // We know that the DE438s has exactly 14 ephemerides.
    assert_eq!(
        ctx.ephemeris_lut.hashes.len(),
        12,
        "DE438s should have 12 ephemerides"
    );

    // For all of the frames in this context, let's make sure that the translation between the same frames is always zero.
    for ephemeris_hash in ctx.ephemeris_lut.hashes.iter() {
        // Build a J2000 oriented frame with this ephemeris center
        let this_frame_j2k = Frame::from_ephem_orient_hashes(*ephemeris_hash, J2000);

        // Check that the common root between the same frame is that frame's hash.
        let root_ephem = ctx
            .find_ephemeris_root(this_frame_j2k, this_frame_j2k)
            .unwrap();

        assert_eq!(root_ephem, *ephemeris_hash);

        // let zero_vec = ctx
        //     .translate_from_to(this_frame_j2k, this_frame_j2k, Epoch::now().unwrap())
        //     .unwrap();
        // dbg!(zero_vec);
    }

    // ctx.lt_translate_from_to(Earth, Moon, epoch, LTCorr) -> position and velocity of the Earth with respect to the Moon with light time correction at epoch
    // ctx.rotate_to_from() -> quaternion
}
