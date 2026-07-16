/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::ops::Range;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::DBL_SIZE;
use log::warn;

use super::{DAFError, NAIFRecord, NAIFSummaryRecord, RCRD_LEN};

#[derive(IntoBytes, FromBytes, KnownLayout, Immutable, Clone, Debug)]
#[repr(C)]
pub struct NameRecord {
    pub(crate) raw_names: [u8; RCRD_LEN],
}

impl Default for NameRecord {
    fn default() -> Self {
        Self {
            raw_names: [0_u8; RCRD_LEN],
        }
    }
}

impl NAIFRecord for NameRecord {}

impl NameRecord {
    /// Returns the maximum number of names in this record given the provided summary size.
    ///
    /// Note that we don't actually use `&self` here, but it's just easier to call.
    pub const fn num_entries(&self, summary_size: usize) -> usize {
        let entry_size = summary_size.saturating_mul(DBL_SIZE);
        if entry_size == 0 {
            // A summary size of zero (nd and ni both zero in the file record) would
            // otherwise divide by zero here when a crafted DAF is queried by name.
            return 0;
        }
        RCRD_LEN / entry_size
    }

    /// Returns the byte range of the n-th entry, or None if it does not fit in this record.
    ///
    /// The summary size is derived from the `nd` and `ni` fields of the file record, which are
    /// read straight from the file, so the range is computed with checked arithmetic and bounded
    /// against the record rather than trusting it to address a valid entry.
    fn entry_range(n: usize, summary_size: usize) -> Option<Range<usize>> {
        let entry_size = summary_size.checked_mul(DBL_SIZE)?;
        let start = n.checked_mul(entry_size)?;
        let end = start.checked_add(entry_size)?;
        if end > RCRD_LEN {
            None
        } else {
            Some(start..end)
        }
    }

    pub fn nth_name(&self, n: usize, summary_size: usize) -> &str {
        let Some(this_name) = Self::entry_range(n, summary_size).map(|rng| &self.raw_names[rng])
        else {
            warn!("malformed name record: entry {n} of {summary_size} words is out of bounds!");
            return "MALFORMED NAME";
        };
        match core::str::from_utf8(this_name) {
            Ok(name) => name.trim(),
            Err(e) => {
                warn!("malformed name record: `{e}` from {this_name:?}!",);
                "MALFORMED NAME"
            }
        }
    }

    /// Changes the name of the n-th record
    pub fn set_nth_name(&mut self, n: usize, summary_size: usize, new_name: &str) {
        let Some(rng) = Self::entry_range(n, summary_size) else {
            warn!("malformed name record: entry {n} of {summary_size} words is out of bounds!");
            return;
        };
        let this_name = &mut self.raw_names[rng];

        // Copy the name (thanks Clippy)
        let cur_len = this_name.len();
        this_name[..new_name.len().min(cur_len)]
            .copy_from_slice(&new_name.as_bytes()[..new_name.len().min(cur_len)]);

        // Set the rest of the data to spaces.
        for mut_char in this_name.iter_mut().skip(new_name.len()) {
            *mut_char = " ".as_bytes()[0];
        }
    }

    /// Searches the name record for the provided name.
    ///
    /// **Warning:** this performs an O(N) search!
    pub fn index_from_name<R: NAIFSummaryRecord>(
        &self,
        name: &str,
        summary_size: usize,
    ) -> Result<usize, DAFError> {
        for i in 0..self.num_entries(summary_size) {
            if self.nth_name(i, summary_size) == name {
                return Ok(i);
            }
        }
        Err(DAFError::NameError {
            kind: R::NAME,
            name: name.to_string(),
        })
    }
}
