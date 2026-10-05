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
use snafu::ensure;

use super::{NoOrientationsLoadedSnafu, OrientationError};
use crate::NaifId;
use crate::almanac::Almanac;
use crate::constants::orientations::{ECLIPJ2000, ICRS, J2000};
use crate::frames::{DynamicFrame, Frame};
use crate::naif::daf::{DAFError, NAIFSummaryRecord};

/// **Limitation:** no translation or rotation may have more than 8 nodes.
pub const MAX_TREE_DEPTH: usize = 8;

impl Almanac {
    /// Returns the root of all of the loaded orientations (BPC or planetary), typically this should be J2000.
    ///
    /// # Algorithm
    ///
    /// 1. For each loaded BPC, iterated in reverse order (to mimic SPICE behavior)
    /// 2. For each summary record in each BPC, follow the orientation branch all the way up until the end of this BPC or until the J2000.
    pub fn try_find_orientation_root(&self) -> Result<NaifId, OrientationError> {
        ensure!(
            self.num_loaded_bpc() > 0 || !self.planetary_data.is_empty(),
            NoOrientationsLoadedSnafu
        );

        // The common center is the absolute minimum of all centers due to the NAIF numbering.
        let mut common_center = i32::MAX;

        for bpc in self.bpc_data.values().rev() {
            for these_summaries in bpc.iter_summary_blocks().flatten() {
                for summary in these_summaries {
                    // This summary exists, so we need to follow the branch of centers up the tree.
                    if !summary.is_empty()
                        && summary.inertial_frame_id.unsigned_abs() < common_center.unsigned_abs()
                    {
                        common_center = summary.inertial_frame_id;
                        if common_center == ICRS {
                            // there is nothing higher up
                            return Ok(common_center);
                        }
                    }
                }
            }
        }

        // If we reached this point, it means that we didn't find J2000 in the loaded BPCs, so let's iterate through the planetary data

        for data in self.planetary_data.values().rev() {
            for id in data.lut.by_id.keys() {
                if let Ok(pc) = data.get_by_id(*id)
                    && pc.parent_id < common_center
                {
                    common_center = pc.parent_id;
                    if common_center == ICRS {
                        // there is nothing higher up
                        return Ok(common_center);
                    }
                }
            }
        }

        if common_center == ECLIPJ2000 || common_center == J2000 {
            // Rotation from ecliptic J2000 / J2000 to ICRS is embedded.
            common_center = ICRS;
        }

        Ok(common_center)
    }

    /// Try to construct the path from the source frame all the way to the root orientation of this context.
    pub fn orientation_path_to_root(
        &self,
        source: Frame,
        epoch: Epoch,
    ) -> Result<(usize, [Option<NaifId>; MAX_TREE_DEPTH]), OrientationError> {
        let common_center = self.try_find_orientation_root()?;
        // Build a tree, set a fixed depth to avoid allocations
        let mut of_path = [None; MAX_TREE_DEPTH];
        let mut of_path_len = 0;

        if common_center == source.orientation_id {
            // We're querying the source, no need to check that this summary even exists.
            return Ok((of_path_len, of_path));
        }

        // Grab the summary data, which we use to find the paths
        // Let's see if this orientation is defined in the loaded BPC files
        let mut inertial_frame_id = if source.orient_origin_id_match(J2000) {
            ICRS
        } else if let Ok(dyn_frame) = DynamicFrame::try_from(source.orientation_id as u32)
            && matches!(
                dyn_frame,
                DynamicFrame::EarthMeanOfDate { .. }
                    | DynamicFrame::EarthTrueOfDate { .. }
                    | DynamicFrame::EarthTrueEquatorMeanEquinox { .. }
            )
        {
            match dyn_frame {
                DynamicFrame::EarthMeanOfDate { precession }
                | DynamicFrame::EarthTrueOfDate { precession, .. }
                | DynamicFrame::EarthTrueEquatorMeanEquinox { precession, .. } => {
                    precession.parent_id()
                }
                _ => unreachable!(),
            }
        } else {
            let orientation_id =
                if let Ok(dyn_frame) = DynamicFrame::try_from(source.orientation_id as u32) {
                    match dyn_frame {
                        DynamicFrame::BodyMeanOfDate { source_id }
                        | DynamicFrame::BodyTrueOfDate { source_id } => source_id,
                        _ => unreachable!("all other variants handled above"),
                    }
                } else {
                    source.orientation_id
                };

            match self.bpc_summary_at_epoch(orientation_id, epoch) {
                Ok((summary, _, _, _)) => summary.inertial_frame_id,
                Err(_) => {
                    // Not available as a BPC, so let's see if there's planetary data for it.
                    match self.get_planetary_data_from_id(orientation_id) {
                        Ok(planetary_data) => planetary_data.parent_id,
                        Err(_) => {
                            // Finally, let's see if it's in the loaded Euler Parameters.
                            self.euler_param_from_id(orientation_id)?.to
                        }
                    }
                }
            }
        };

        of_path[of_path_len] = Some(inertial_frame_id);
        of_path_len += 1;

        // Hop the embedded constant rotations up to the common root.
        // Future embedded constant-rotation parents should be added here.
        if inertial_frame_id == ECLIPJ2000 || inertial_frame_id == J2000 {
            inertial_frame_id = ICRS;
            of_path[of_path_len] = Some(inertial_frame_id);
            of_path_len += 1;
        }

        if inertial_frame_id == common_center {
            // Well that was quick!
            return Ok((of_path_len, of_path));
        }

        for _ in 0..MAX_TREE_DEPTH - 1 {
            inertial_frame_id = match self.bpc_summary_at_epoch(inertial_frame_id, epoch) {
                Ok((summary, _, _, _)) => summary.inertial_frame_id,
                Err(_) => {
                    // Not available as a BPC, so let's see if there's planetary data for it.
                    match self.get_planetary_data_from_id(inertial_frame_id) {
                        Ok(planetary_data) => planetary_data.parent_id,
                        Err(_) => {
                            // Finally, let's see if it's in the loaded Euler Parameters.
                            self.euler_param_from_id(inertial_frame_id)?.to
                        }
                    }
                }
            };

            if of_path_len >= MAX_TREE_DEPTH {
                return Err(OrientationError::BPC {
                    action: "computing path to common node",
                    source: DAFError::MaxRecursionDepth,
                });
            }
            of_path[of_path_len] = Some(inertial_frame_id);
            of_path_len += 1;
            if inertial_frame_id == common_center {
                // We're found the path!
                return Ok((of_path_len, of_path));
            }
        }

        Err(OrientationError::BPC {
            action: "computing path to common node",
            source: DAFError::MaxRecursionDepth,
        })
    }

    /// Returns the orientation path between two frames and the common node. This may return a `DisjointRoots` error if the frames do not share a common root, which is considered a file integrity error.
    pub fn common_orientation_path(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<(usize, [Option<NaifId>; MAX_TREE_DEPTH], NaifId), OrientationError> {
        if from_frame == to_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok((0, [None; MAX_TREE_DEPTH], from_frame.orientation_id));
        }

        // Grab the paths
        let (from_len, from_path) = self.orientation_path_to_root(from_frame, epoch)?;
        let (to_len, to_path) = self.orientation_path_to_root(to_frame, epoch)?;

        // Now that we have the paths, we can find the matching origin.

        // If either path is of zero length, that means one of them is at the root of this ANISE file, so the common
        // path is which brings the non zero-length path back to the file root.
        if from_len == 0 && to_len == 0 {
            Err(OrientationError::RotationOrigin {
                from: from_frame.into(),
                to: to_frame.into(),
                epoch,
            })
        } else if from_len != 0 && to_len == 0 {
            // One has an empty path but not the other, so the root is at the empty path
            Ok((from_len, from_path, to_frame.orientation_id))
        } else if to_len != 0 && from_len == 0 {
            // One has an empty path but not the other, so the root is at the empty path
            Ok((to_len, to_path, from_frame.orientation_id))
        } else {
            // Either are at the orientation root, so we'll step through the paths until we find the common root.
            let mut common_path = to_path;
            let mut items: usize = to_len;
            let common_node = to_path[to_len - 1].expect("path entry within to_len must be Some");

            for from_obj in from_path.iter().take(from_len).rev().skip(1) {
                // `items` starts at `to_len` and grows with the from path, so it can run past
                // the fixed-size common-path buffer when both frames sit in deep branches.
                // Rather than write out of bounds, report the tree as too deep.
                if items >= common_path.len() {
                    return Err(OrientationError::BPC {
                        action: "computing common path between frames",
                        source: DAFError::MaxRecursionDepth,
                    });
                }
                common_path[items] =
                    Some(from_obj.expect("path entry within from_len must be Some"));
                items += 1;
            }

            Ok((items, common_path, common_node))
        }
    }
}

#[cfg(test)]
mod orientation_root_ut {
    use crate::almanac::Almanac;
    use crate::naif::daf::{FileRecord, SummaryRecord};
    use crate::naif::pck::BPCSummaryRecord;
    use crate::prelude::BPC;
    use zerocopy::IntoBytes;

    #[test]
    fn orientation_root_i32_min_frame_does_not_panic() {
        // A summary's inertial_frame_id is read verbatim from the file, and
        // try_find_orientation_root compares magnitudes with `inertial_frame_id.abs()`.
        // i32::MIN has no positive counterpart, so `.abs()` overflows and panics under
        // overflow checks on a crafted BPC.
        let mut file_record = FileRecord::bpc("IMIN");
        file_record.forward = 2;
        file_record.nd = 2;
        file_record.ni = 5;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(file_record.as_bytes());
        bytes.resize(1024, 0);

        let header = SummaryRecord {
            next_record: 0.0,
            prev_record: 0.0,
            num_summaries: 1.0,
        };
        let mut summary_block = Vec::new();
        summary_block.extend_from_slice(header.as_bytes());
        let summary = BPCSummaryRecord {
            start_epoch_et_s: -1e9,
            end_epoch_et_s: 1e9,
            frame_id: 3000,
            inertial_frame_id: i32::MIN,
            data_type_i: 2,
            start_idx: 1,
            end_idx: 100,
            unused: 0,
        };
        summary_block.extend_from_slice(summary.as_bytes());
        summary_block.resize(1024, 0);
        bytes.extend(summary_block);
        bytes.extend(vec![0u8; 1024]);

        let bpc = BPC::parse(&bytes[..]).unwrap();
        let almanac = Almanac::from_bpc(bpc);

        // Must not panic while scanning the summaries for the root.
        assert!(almanac.try_find_orientation_root().is_ok());
    }
}
