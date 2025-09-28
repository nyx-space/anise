/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::ScalarExpr;
use hifitime::Duration;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Defines a state parameter event finder
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// The state parameter
    pub scalar: ScalarExpr,
    /// The desired self.desired_value, must be in the same units as the state parameter
    pub desired_value: f64,
    /// The duration precision after which the solver will report that it cannot find any more precise
    pub epoch_precision: Duration,
    /// The precision on the desired value
    pub value_precision: f64,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.scalar)?;
        if self.desired_value.abs() > 1e3 {
            write!(
                f,
                " = {:e} (± {:e})",
                self.desired_value, self.value_precision,
            )
        } else {
            write!(f, " = {} (± {})", self.desired_value, self.value_precision,)
        }
    }
}
