/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;
use snafu::{ensure, ResultExt};

use super::{EphemerisError, NoEphemerisLoadedSnafu, SPKSnafu};
use crate::almanac::Almanac;
use crate::frames::Frame;
use crate::naif::daf::{DAFError, NAIFSummaryRecord};
use crate::NaifId;

/// **Limitation:** no translation or rotation may have more than 8 nodes.
pub const MAX_TREE_DEPTH: usize = 8;

impl Almanac {
    /// Returns the root of all of the loaded ephemerides, typically this should be the Solar System Barycenter.
    ///
    /// # Algorithm
    ///
    /// 1. For each loaded SPK, iterated in reverse order (to mimic SPICE behavior)
    /// 2. For each summary record in each SPK, follow the ephemeris branch all the way up until the end of this SPK or until the SSB.
    pub fn try_find_ephemeris_root(&self) -> Result<NaifId, EphemerisError> {
        ensure!(self.num_loaded_spk() > 0, NoEphemerisLoadedSnafu);

        // The common center is the absolute minimum of all centers due to the NAIF numbering.
        let mut common_center = i32::MAX;

        for maybe_spk in self.spk_data.iter().take(self.num_loaded_spk()).rev() {
            let spk = maybe_spk.as_ref().unwrap();

            for summary in spk.data_summaries().context(SPKSnafu {
                action: "finding ephemeris root",
            })? {
                // This summary exists, so we need to follow the branch of centers up the tree.
                if !summary.is_empty() && summary.center_id.abs() < common_center.abs() {
                    common_center = summary.center_id;
                    if common_center == 0 {
                        // We're at the SSB, there is nothing higher up
                        return Ok(common_center);
                    }
                }
            }
        }
        Ok(common_center)
    }

    /// Try to construct the path from the source frame all the way to the root ephemeris of this context.
    pub fn ephemeris_path_to_root(
        &self,
        source: Frame,
        epoch: Epoch,
    ) -> Result<(usize, [Option<NaifId>; MAX_TREE_DEPTH]), EphemerisError> {
        let common_center = self.try_find_ephemeris_root()?;
        // Build a tree, set a fixed depth to avoid allocations
        let mut of_path = [None; MAX_TREE_DEPTH];
        let mut of_path_len = 0;

        if common_center == source.ephemeris_id {
            // We're querying the source, no need to check that this summary even exists.
            return Ok((of_path_len, of_path));
        }

        // Grab the summary data, which we use to find the paths
        let summary = self.spk_summary_at_epoch(source.ephemeris_id, epoch)?.0;

        let mut center_id = summary.center_id;

        of_path[of_path_len] = Some(summary.center_id);
        of_path_len += 1;

        if summary.center_id == common_center {
            // Well that was quick!
            return Ok((of_path_len, of_path));
        }

        for _ in 0..MAX_TREE_DEPTH {
            let summary = self.spk_summary_at_epoch(center_id, epoch)?.0;
            center_id = summary.center_id;
            of_path[of_path_len] = Some(center_id);
            of_path_len += 1;
            if center_id == common_center {
                // We're found the path!
                return Ok((of_path_len, of_path));
            }
        }

        Err(EphemerisError::SPK {
            action: "computing path to common node",
            source: DAFError::MaxRecursionDepth,
        })
    }

    /// Returns the ephemeris path between two frames and the common node. This may return a `DisjointRoots` error if the frames do not share a common root, which is considered a file integrity error.
    ///
    /// # Example
    ///
    /// If the "from" frame is _Earth Barycenter_ whose path to the ANISE root is the following:
    /// ```text
    /// Solar System barycenter
    /// ╰─> Earth Moon Barycenter
    ///     ╰─> Earth
    /// ```
    ///
    /// And the "to" frame is _Moon_, whose path is:
    /// ```text
    /// Solar System barycenter
    /// ╰─> Earth Moon Barycenter
    ///     ╰─> Moon
    ///         ╰─> LRO
    /// ```
    ///
    /// Then this function will return the path an array of hashes of up to [MAX_TREE_DEPTH] items. In this example, the array with the hashes of the "Earth Moon Barycenter" and "Moon".
    ///
    /// # Note
    /// A proper ANISE file should only have a single root and if two paths are empty, then they should be the same frame.
    /// If a DisjointRoots error is reported here, it means that the ANISE file is invalid.
    ///
    /// # Time complexity
    /// This can likely be simplified as this as a time complexity of O(n×m) where n, m are the lengths of the paths from
    /// the ephemeris up to the root.
    /// This can probably be optimized to avoid rewinding the entire frame path up to the root frame
    pub fn common_ephemeris_path(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<(usize, [Option<NaifId>; MAX_TREE_DEPTH], NaifId), EphemerisError> {
        if from_frame == to_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok((0, [None; MAX_TREE_DEPTH], from_frame.ephemeris_id));
        }

        // Grab the paths
        let (from_len, from_path) = self.ephemeris_path_to_root(from_frame, epoch)?;
        let (to_len, to_path) = self.ephemeris_path_to_root(to_frame, epoch)?;

        // Now that we have the paths, we can find the matching origin.

        // If either path is of zero length, that means one of them is at the root of this ANISE file, so the common
        // path is which brings the non zero-length path back to the file root.
        if from_len == 0 && to_len == 0 {
            Err(EphemerisError::TranslationOrigin {
                from: from_frame.into(),
                to: to_frame.into(),
                epoch,
            })
        } else if from_len != 0 && to_len == 0 {
            // One has an empty path but not the other, so the root is at the empty path
            Ok((from_len, from_path, to_frame.ephemeris_id))
        } else if to_len != 0 && from_len == 0 {
            // One has an empty path but not the other, so the root is at the empty path
            Ok((to_len, to_path, from_frame.ephemeris_id))
        } else {
            // Either are at the ephemeris root, so we'll step through the paths until we find the common root.
            let mut common_path = [None; MAX_TREE_DEPTH];
            let mut items: usize = 0;

            for to_obj in to_path.iter().take(to_len) {
                // Check the trivial case of the common node being one of the input frames
                if to_obj.unwrap() == from_frame.ephemeris_id {
                    common_path[0] = Some(from_frame.ephemeris_id);
                    items = 1;
                    return Ok((items, common_path, from_frame.ephemeris_id));
                }

                for from_obj in from_path.iter().take(from_len) {
                    // Check the trivial case of the common node being one of the input frames
                    if items == 0 && from_obj.unwrap() == to_frame.ephemeris_id {
                        common_path[0] = Some(to_frame.ephemeris_id);
                        items = 1;
                        return Ok((items, common_path, to_frame.ephemeris_id));
                    }

                    common_path[items] = Some(from_obj.unwrap());
                    items += 1;

                    if from_obj == to_obj {
                        // This is where the paths branch meet, so the root is the parent of the current item.
                        // Recall that the path is _from_ the source to the root of the context, so we're walking them
                        // backward until we find "where" the paths branched out.
                        return Ok((items, common_path, to_obj.unwrap()));
                    }
                }
            }

            // This is weird and I don't think it should happen, so let's raise an error.
            Err(EphemerisError::Unreachable)
        }
    }
}
