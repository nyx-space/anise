/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::analysis::{event::Event, ScalarExpr, StateSpec};
use serde::{Deserialize, Serialize};

/// A basic report builder that can be serialized seperately from the execution.
// TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed, enable Dhall serialization.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReportScalars {
    pub scalars: Vec<(ScalarExpr, Option<String>)>,
    pub state_spec: StateSpec,
}

impl ReportScalars {
    /// Export this Scalar Expression to S-Expression / LISP syntax
    pub fn to_s_expr(&self) -> String {
        serde_lexpr::to_value(&self).unwrap().to_string()
    }

    /// Load this Scalar Expression from an S-Expression / LISP syntax
    pub fn from_s_expr(expr: &str) -> Result<Self, serde_lexpr::Error> {
        serde_lexpr::from_str(expr)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReportEvents {
    pub scalars: Vec<(Event, Option<String>)>,
    pub state_spec: StateSpec,
}

impl ReportEvents {
    /// Export this Scalar Expression to S-Expression / LISP syntax
    pub fn to_s_expr(&self) -> String {
        serde_lexpr::to_value(&self).unwrap().to_string()
    }

    /// Load this Scalar Expression from an S-Expression / LISP syntax
    pub fn from_s_expr(expr: &str) -> Result<Self, serde_lexpr::Error> {
        serde_lexpr::from_str(expr)
    }
}
