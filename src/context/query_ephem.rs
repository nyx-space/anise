/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::{error, trace};

use crate::asn1::units::*;
use crate::constants::orientations::J2000;
use crate::errors::InternalErrorKind;
use crate::hifitime::Epoch;
use crate::math::{Aberration, Vector3};
use crate::HashType;
use crate::{
    asn1::{context::AniseContext, ephemeris::Ephemeris},
    errors::{AniseError, IntegrityErrorKind},
    frame::Frame,
};

/// **Limitation:** no translation or rotation may have more than 8 nodes.
pub const MAX_TREE_DEPTH: usize = 8;

impl<'a> AniseContext<'a> {
    /// Goes through each ephemeris data and make sure that the root of each is the same.
    /// A context is only valid if the data is a tree with a single top level root.
    pub fn try_find_context_root(&self) -> Result<HashType, AniseError> {
        let mut common_parent_hash = 0;
        for e in self.ephemeris_data.iter() {
            let mut child = e;
            if common_parent_hash == 0 {
                common_parent_hash = child.parent_ephemeris_hash;
            }

            for _ in 0..MAX_TREE_DEPTH {
                match self.try_find_parent(child) {
                    Ok(e) => child = e,
                    Err(AniseError::ItemNotFound) => {
                        // We've found the end of this branch, so let's store the parent of the child as the top root if the top root is not set
                        if common_parent_hash == 0 {
                            common_parent_hash = child.parent_ephemeris_hash;
                        } else if common_parent_hash != child.parent_ephemeris_hash {
                            // Integrity error!
                            error!("at least one ephemeris hierarchy takes root in hash {} but {}'s parent is {}", common_parent_hash, child.name, child.parent_ephemeris_hash);
                            return Err(AniseError::IntegrityError(
                                IntegrityErrorKind::DisjointRoots {
                                    from_frame: Frame::from_ephem_orient(common_parent_hash, J2000),
                                    to_frame: Frame::from_ephem_orient(
                                        child.parent_ephemeris_hash,
                                        J2000,
                                    ),
                                },
                            ));
                        } else {
                            // We're found the root and it matches the previous one, so we can stop searching.
                            return Ok(common_parent_hash);
                            // break;
                        }
                    }
                    Err(err) => {
                        error!("{err} occurred when it should not have");
                        return Err(AniseError::InternalError(InternalErrorKind::Generic));
                    }
                };
            }
        }
        // return Ok(common_parent_hash);
        Err(AniseError::MaxTreeDepth)
    }

    /// Try to find the parent ephemeris data of the provided ephemeris.
    ///
    /// Will return an [AniseError] if the parent does not have ephemeris data in this context.
    pub fn try_find_parent(&self, child: &'a Ephemeris) -> Result<&'a Ephemeris, AniseError> {
        let idx = self
            .ephemeris_lut
            .index_for_hash(&child.parent_ephemeris_hash)?;
        self.try_ephemeris_data(idx.into())
    }

    /// Try to return the ephemeris for the provided index, or returns an error.
    pub fn try_ephemeris_data(&self, idx: usize) -> Result<&'a Ephemeris, AniseError> {
        self.ephemeris_data
            .get(idx)
            .ok_or(AniseError::IntegrityError(IntegrityErrorKind::LookupTable))
    }

    /// Try to return the orientation for the provided index, or returns an error.
    pub fn try_orientation_data(&self, idx: usize) -> Result<&'a Ephemeris, AniseError> {
        self.orientation_data
            .get(idx)
            .ok_or(AniseError::IntegrityError(IntegrityErrorKind::LookupTable))
    }

    /// Try to construct the path from the source frame all the way to the root ephemeris of this context.
    pub fn try_ephemeris_path(
        &self,
        source: &Frame,
    ) -> Result<(usize, [Option<HashType>; MAX_TREE_DEPTH]), AniseError> {
        // Build a tree, set a fixed depth to avoid allocations
        let mut of_path = [None; MAX_TREE_DEPTH];
        let mut of_path_len = 0;
        let mut prev_ephem_hash = source.ephemeris_hash;

        for _ in 0..MAX_TREE_DEPTH {
            let idx = self.ephemeris_lut.index_for_hash(&prev_ephem_hash)?;
            let parent_ephem = self.try_ephemeris_data(idx.into())?;
            let parent_hash = parent_ephem.parent_ephemeris_hash;
            of_path[of_path_len] = Some(parent_hash);
            of_path_len += 1;

            if parent_hash == self.try_find_context_root()? {
                return Ok((of_path_len, of_path));
            } else if let Err(e) = self.ephemeris_lut.index_for_hash(&parent_hash) {
                if e == AniseError::ItemNotFound {
                    // We have reached the root of this ephemeris and it has no parent.
                    trace!("{parent_hash} has no parent in this context");
                    return Ok((of_path_len, of_path));
                }
            }
            prev_ephem_hash = parent_hash;
        }
        Err(AniseError::MaxTreeDepth)
    }

    /// Returns the common node of two frames. This may return a `DisjointRoots` error if the frames do not share a common root, which is considered a file integrity error.
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
    /// And the "to" frame is _Luna_, whose path is:
    /// ```text
    /// Solar System barycenter
    /// ╰─> Earth Moon Barycenter
    ///     ╰─> Luna
    ///         ╰─> LRO
    /// ```
    ///
    /// Then this function will return the common root/node as a hash, in this case, the hash of the "Earth Moon Barycenter".
    ///
    /// # Note
    /// A proper ANISE file should only have a single root and if two paths are empty, then they should be the same frame.
    /// If a DisjointRoots error is reported here, it means that the ANISE file is invalid.
    ///
    /// # Time complexity
    /// This can likely be simplified as this as a time complexity of O(n×m) where n, m are the lengths of the paths from
    /// the ephemeris up to the root.
    pub fn find_common_ephemeris_node(
        &self,
        from_frame: Frame,
        to_frame: Frame,
    ) -> Result<HashType, AniseError> {
        if from_frame == to_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok(from_frame.ephemeris_hash);
        }

        // Grab the paths
        let (from_len, from_path) = self.try_ephemeris_path(&from_frame)?;
        let (to_len, to_path) = self.try_ephemeris_path(&to_frame)?;

        // Now that we have the paths, we can find the matching origin.

        // If either path is of zero length, that means one of them is at the root of this ANISE file, so the common
        // path is which brings the non zero-length path back to the file root.
        if from_len == 0 && to_len == 0 {
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::DisjointRoots {
                    from_frame,
                    to_frame,
                },
            ))
        } else if from_len != 0 && to_len == 0 {
            // One has an empty path but not the other, so the root is at the empty path
            Ok(to_frame.ephemeris_hash)
        } else if to_len != 0 && from_len == 0 {
            // One has an empty path but not the other, so the root is at the empty path
            Ok(from_frame.ephemeris_hash)
        } else {
            // Either are at the ephemeris root, so we'll step through the paths until we find the common root.
            if from_len > to_len {
                // Iterate through the items in to_path because the longest path is necessarily includes in the shorter one,
                // so we can shrink the outer loop here
                for to_obj in to_path.iter().take(to_len) {
                    for from_obj in from_path.iter().take(from_len) {
                        if from_obj == to_obj {
                            // This is where the paths branch meet, so the root is the parent of the current item.
                            // Recall that the path is _from_ the source to the root of the context, so we're walking them
                            // backward until we find "where" the paths branched out.
                            return Ok(to_obj.unwrap());
                        }
                    }
                }
            } else {
                // Same algorithm as above, just flipped to make sure the outer loop is the shorter one
                for from_obj in from_path.iter().take(from_len) {
                    for to_obj in to_path.iter().take(to_len) {
                        if from_obj == to_obj {
                            // This is where the paths branch meet, so the root is the parent of the current item.
                            // Recall that the path is _from_ the source to the root of the context, so we're walking them
                            // backward until we find "where" the paths branched out.
                            return Ok(to_obj.unwrap());
                        }
                    }
                }
            }
            // This is weird and I don't think it should happen, so let's raise an error.
            Err(AniseError::IntegrityError(IntegrityErrorKind::DataMissing))
        }
    }

    /// Returns the position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`.
    ///
    /// **WARNING:** This function only performs the translation and no rotation whatsoever. Use the `transform_from_to` function instead to include rotations.
    ///
    /// Note: this function performs a recursion of no more than twice the [MAX_TREE_DEPTH].
    pub fn translate_from_to(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
        distance_unit: DistanceUnit,
        time_unit: TimeUnit,
    ) -> Result<(Vector3, Vector3, Vector3), AniseError> {
        if from_frame == to_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok((Vector3::zeros(), Vector3::zeros(), Vector3::zeros()));
        }

        let ephem_root = self.find_common_ephemeris_node(from_frame, to_frame)?;

        // Compute from the center back to its origin and then translate from that origin to the

        // Now that we have the root, let's simply add the vectors from each frame to the root.

        let (pos_from_to_root, vel_from_to_root, accel_from_to_root) = self.translate_from_to(
            from_frame,
            from_frame.with_ephem(ephem_root),
            epoch,
            ab_corr,
            distance_unit,
            time_unit,
        )?;

        let (pos_to_to_root, vel_to_to_root, accel_to_to_root) = self.translate_from_to(
            to_frame,
            to_frame.with_ephem(ephem_root),
            epoch,
            ab_corr,
            distance_unit,
            time_unit,
        )?;

        // Return the difference of both vectors.
        Ok((
            pos_from_to_root - pos_to_to_root,
            vel_from_to_root - vel_to_to_root,
            accel_from_to_root - accel_to_to_root,
        ))
    }

    /// Returns the position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2.
    pub fn translate_from_to_km_s(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
    ) -> Result<(Vector3, Vector3, Vector3), AniseError> {
        self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            ab_corr,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
    }

    /// Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations.
    pub fn translate_state_to(
        &self,
        position: Vector3,
        velocity: Vector3,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
        distance_unit: DistanceUnit,
        time_unit: TimeUnit,
    ) -> Result<(Vector3, Vector3), AniseError> {
        // Compute the frame translation
        let (frame_pos, frame_vel, _) = self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            ab_corr,
            distance_unit,
            time_unit,
        )?;

        Ok((position + frame_pos, velocity + frame_vel))
    }
}
