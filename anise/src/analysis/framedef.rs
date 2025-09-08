/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use super::expr::VectorExpr;

// Defines how to build an orthogonal frame from custom vector expressions
#[derive(Debug)]
#[cfg_attr(not(feature = "python"), Clone, PartialEq)]
pub enum OrthonormalFrame {
    CrossProductXY { x: VectorExpr, y: VectorExpr },
    CrossProductXZ { x: VectorExpr, z: VectorExpr },
    CrossProductYZ { y: VectorExpr, z: VectorExpr },
}

/// Defines a runtime reference frame from an orthogonal frame, allowing it to be normalized or some vectors negated.
/// Note that if trying to negate a vector that isn't used in the definition of the orthogonal frame, an error will be raised.
#[derive(Debug)]
#[cfg_attr(not(feature = "python"), Clone, PartialEq)]
pub struct CustomFrameDef {
    pub frame: OrthonormalFrame,
    pub negate_x: bool,
    pub negate_y: bool,
    pub negate_z: bool,
}
