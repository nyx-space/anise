/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{compare::*, validate::Validation};
use anise::almanac::metaload::MetaFile;
use std::env;

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_lagrange_type9_with_varying_segment_sizes() {
    if let Err(_) = env::var("LAGRANGE_BSP") {
        // Skip this test if the env var is not defined.
        return;
    }

    let mut lagrange_meta = MetaFile {
        uri: "http://public-data.nyxspace.com/anise/ci/env:LAGRANGE_BSP".to_string(),
        crc32: None,
    };
    lagrange_meta.process(true).unwrap();

    let file_name = "spk-type9-validation-variable-seg-size".to_string();
    let comparator = CompareEphem::new(vec![lagrange_meta.uri], file_name.clone(), 10_000, None);

    let err_count = comparator.run();

    assert_eq!(err_count, 0, "None of the queries should fail!");

    let validator = Validation {
        file_name,
        max_q75_err: 5e-9,
        max_q99_err: 2e-7,
        max_abs_err: 0.05,
    };

    validator.validate();
}
