/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use anise::prelude::*;

#[test]
fn test_de438s_load() {
    let filename = "data/de440.bsp";
    let bytes = file_mmap!(filename).unwrap();
    let mut hdr_end_idx = 0;
    loop {
        if bytes[hdr_end_idx] == 0x0 {
            break;
        }
        hdr_end_idx += 1;
    }
    assert_eq!(std::str::from_utf8(&bytes[0..8]).unwrap(), "DAF/SPK ");
    dbg!(hdr_end_idx);

    // const RCRD_SIZE: usize = 1024;

    // for i in 0..10 {
    //     println!("{:?}", &bytes[i * RCRD_SIZE..(i + 1) * RCRD_SIZE],)
    // }

    println!("{:?}", &bytes[0..8])
}
