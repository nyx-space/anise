/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::{collections::HashMap, hash::Hash};

use self::parser::Assignment;

#[cfg(feature = "std")]
pub mod parser;
pub mod tpc;

pub trait KPLItem: Default {
    type Parameter: Eq + Hash;
    /// The key used for fetching
    fn extract_key(keyword: &str) -> i32;
    fn data(&self) -> &HashMap<Self::Parameter, Vec<f64>>;
    fn parse(&mut self, data: Assignment);
}
