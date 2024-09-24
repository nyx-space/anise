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
fn validate_jplde_de440_type3_no_aberration() {
    let file_name = "spk-type3-validation-de440".to_string();
    let comparator = CompareEphem::new(
        vec!["../data/de440_type3.bsp".to_string()],
        file_name.clone(),
        1_000,
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
