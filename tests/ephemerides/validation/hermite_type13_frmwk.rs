/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{compare::*, validate::Validation};

#[test]
fn validate_hermite_type13_from_gmat() {
    let comparator = CompareEphem::new(
        vec![
            // "data/de440.bsp".to_string(),
            "data/gmat-hermite.bsp".to_string(),
        ],
        "type13-validation-test-results".to_string(),
    );

    comparator.run();

    let validator = Validation {
        file_name: "type13-validation-test-results".to_string(),
    };

    validator.validate();
}
