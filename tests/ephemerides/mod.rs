/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

mod parent_translation_verif;
mod paths;
mod translation;
#[cfg(feature = "validation")]
mod validation;

#[allow(dead_code)]
pub mod consts {
    pub const MAX_ABS_POS_ERR_KM: f64 = 10.0; // Absolute error (absolute difference regardless of the scale of the numbers)
    pub const MAX_ABS_VEL_ERR_KM_S: f64 = 1e-3;
    pub const MAX_REL_POS_ERR_KM: f64 = 1e-5; // Relative error is the scaled error
    pub const TYPICAL_REL_POS_ERR_KM: f64 = 1e-7;
    pub const MAX_REL_VEL_ERR_KM_S: f64 = 1e-3;
    pub const TYPICAL_REL_VEL_ERR_KM_S: f64 = 1e-8;
}
