/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::analysis::{ScalarExpr, StateSpec};
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::types::PyType;

#[cfg(feature = "python")]
use super::python::{PyScalarExpr, PyStateSpec};

/// A basic report builder that can be serialized seperately from the execution.
/// The scalars must be a tuple of (ScalarExpr, String) where the String is the alias (optional).
// TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed, enable Dhall serialization.
//
// :type scalars: list
// :type state_spec: StateSpec
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis"))]
pub struct ReportScalars {
    pub scalars: Vec<(ScalarExpr, Option<String>)>,
    pub state_spec: StateSpec,
}

impl ReportScalars {
    /// Export this Scalar Expression to S-Expression / LISP syntax
    pub fn to_s_expr(&self) -> Result<String, serde_lexpr::Error> {
        Ok(serde_lexpr::to_value(self)?.to_string())
    }

    /// Load this Scalar Expression from an S-Expression / LISP syntax
    pub fn from_s_expr(expr: &str) -> Result<Self, serde_lexpr::Error> {
        serde_lexpr::from_str(expr)
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl ReportScalars {
    /// Convert the S-Expression to a report builder
    /// :type expr: str
    /// :rtype: Event
    #[classmethod]
    #[pyo3(name = "from_s_expr")]
    fn py_from_s_expr(_cls: Bound<'_, PyType>, expr: &str) -> Result<Self, PyErr> {
        Self::from_s_expr(expr).map_err(|e| PyException::new_err(e.to_string()))
    }

    /// Converts this report builder to its S-Expression
    /// :rtype: str
    #[pyo3(name = "to_s_expr")]
    fn py_to_s_expr(&self) -> Result<String, PyErr> {
        self.to_s_expr()
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[new]
    fn new(scalars: Vec<(PyScalarExpr, Option<String>)>, state_spec: PyStateSpec) -> Self {
        let state_spec = StateSpec::from(state_spec);

        let scalars = scalars
            .into_iter()
            .map(|(scalar, opt_alias)| (ScalarExpr::from(scalar), opt_alias))
            .collect();

        Self {
            scalars,
            state_spec,
        }
    }
}
