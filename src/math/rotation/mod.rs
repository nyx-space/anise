/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/// The smallest difference between two radians is set to one milliarcsecond, or about 2.8e-7 degrees.
pub const EPSILON_RAD: f64 = 4.8e-9;

mod dcm;
mod mrp;
mod quaternion;
pub use dcm::DCM;
pub use mrp::MRP;
pub use quaternion::Quaternion;

pub trait Rotation: TryInto<Quaternion> {}
