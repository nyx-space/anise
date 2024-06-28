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

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_hermite_type13_from_gmat() {
    let file_name = "spk-type13-validation-even-seg-size".to_string();
    let comparator = CompareEphem::new(
        vec!["../data/gmat-hermite.bsp".to_string()],
        file_name.clone(),
        10_000,
        None,
    );

    let err_count = comparator.run();

    assert_eq!(err_count, 0, "None of the queries should fail!");

    let validator = Validation {
        file_name,
        ..Default::default()
    };

    validator.validate();
}

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_hermite_type13_with_varying_segment_sizes() {
    // ISSUE: This file is corrupt, cf. https://github.com/nyx-space/anise/issues/262
    let file_name = "spk-type13-validation-variable-seg-size".to_string();
    let comparator = CompareEphem::new(
        vec!["../data/variable-seg-size-hermite.bsp".to_string()],
        file_name.clone(),
        10_000,
        None,
    );

    let err_count = comparator.run();

    assert_eq!(err_count, 0, "None of the queries should fail!");

    // BUG: For variable sized Type 13, there is an error at the very end of the file.
    let validator = Validation {
        file_name,
        max_q75_err: 5e-9,
        max_q99_err: 2e-7,
        max_abs_err: 0.05,
    };

    validator.validate();
}
