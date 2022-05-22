/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::der::Enumerated;

#[derive(Enumerated, Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InterpolationKind {
    ChebyshevSeries = 0,
    HermiteSeries = 1,
    LagrangeSeries = 2,
    Polynomial = 3,
    Trigonometric = 4, // Sometimes called Fourier Series interpolation
}
