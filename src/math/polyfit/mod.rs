/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::ops::{Index, IndexMut};

/// A FixedArray is a a way around allocating vectors when we don't know the exact size at compile time.
/// This will be made obsolete when https://github.com/rust-lang/rust/issues/60551 is merged into rust stable.
#[derive(Copy, Clone, Debug)]
pub struct F64TupleArray<const N: usize, const S: usize>([[f64; N]; S]);

impl<const N: usize, const S: usize> F64TupleArray<N, S> {
    pub fn zeros() -> Self {
        Self([[0.0; N]; S])
    }

    const fn indexes(&self, index: usize) -> (usize, usize) {
        (index / N, index % N)
    }
}

impl<const N: usize, const S: usize> Index<usize> for F64TupleArray<N, S> {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        let (one, two) = self.indexes(index);
        &self.0[one][two]
    }
}

impl<const N: usize, const S: usize> IndexMut<usize> for F64TupleArray<N, S> {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        let (one, two) = self.indexes(index);
        &mut self.0[one][two]
    }
}
