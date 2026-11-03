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
fn validate_modified_diff_type21_ceres() {
    let file_name = "spk-type21-validation-ext-mod-diff".to_string();
    let comparator = CompareEphem::new(
        vec!["../data/ceres_horizons_type21.bsp".to_string()],
        file_name.clone(),
        10_000,
        None,
    );

    let err_count = comparator.run();

    assert_eq!(err_count, 0, "None of the queries should fail!");

    let validator = Validation {
        file_name,
        max_q75_err: 1e-12,
        max_q99_err: 1e-14,
        max_abs_err: 2e-7,
        ..Default::default()
    };

    validator.validate();
}
