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
    pub fn ephemeris_path_to_root(
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
    /// And the "to" frame is _Luna_, whose path is:
    /// ```text
    /// Solar System barycenter
    /// ╰─> Earth Moon Barycenter
    ///     ╰─> Luna
    ///         ╰─> LRO
    /// ```
    ///
    /// Then this function will return the path an array of hashes of up to [MAX_TREE_DEPTH] items. In this example, the array with the hashes of the "Earth Moon Barycenter" and "Luna".
    ///
    /// # Note
    /// A proper ANISE file should only have a single root and if two paths are empty, then they should be the same frame.
    /// If a DisjointRoots error is reported here, it means that the ANISE file is invalid.
    ///
    /// # Time complexity
    /// This can likely be simplified as this as a time complexity of O(n×m) where n, m are the lengths of the paths from
    /// the ephemeris up to the root.
    pub fn common_ephemeris_path(
        &self,
        from_frame: Frame,
        to_frame: Frame,
    ) -> Result<(usize, [Option<HashType>; MAX_TREE_DEPTH], HashType), AniseError> {
        // TODO: Consider returning a structure that has explicit fields -- see how I use it first
        if from_frame == to_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok((0, [None; MAX_TREE_DEPTH], from_frame.ephemeris_hash));
        }

        // Grab the paths
        let (from_len, from_path) = self.ephemeris_path_to_root(&from_frame)?;
        let (to_len, to_path) = self.ephemeris_path_to_root(&to_frame)?;

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
            Ok((from_len, from_path, to_frame.ephemeris_hash))
        } else if to_len != 0 && from_len == 0 {
            // One has an empty path but not the other, so the root is at the empty path
            Ok((to_len, to_path, from_frame.ephemeris_hash))
        } else {
            // Either are at the ephemeris root, so we'll step through the paths until we find the common root.
            let mut common_path = [None; MAX_TREE_DEPTH];
            let mut items: usize = 0;

            for to_obj in to_path.iter().take(to_len) {
                // Check the trivial case of the common node being one of the input frames
                if to_obj.unwrap() == from_frame.ephemeris_hash {
                    common_path[0] = Some(from_frame.ephemeris_hash);
                    items = 1;
                    return Ok((items, common_path, from_frame.ephemeris_hash));
                }

                for from_obj in from_path.iter().take(from_len) {
                    // Check the trivial case of the common node being one of the input frames
                    if items == 0 && from_obj.unwrap() == to_frame.ephemeris_hash {
                        common_path[0] = Some(to_frame.ephemeris_hash);
                        items = 1;
                        return Ok((items, common_path, to_frame.ephemeris_hash));
                    }

                    if from_obj == to_obj {
                        // This is where the paths branch meet, so the root is the parent of the current item.
                        // Recall that the path is _from_ the source to the root of the context, so we're walking them
                        // backward until we find "where" the paths branched out.
                        trace!("common path: {common_path:?}");
                        return Ok((items, common_path, to_obj.unwrap()));
                    } else {
                        common_path[items] = Some(from_obj.unwrap());
                        items += 1;
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

        let (node_count, path, common_node) = self.common_ephemeris_path(to_frame, from_frame)?;

        // The fwrd variables are the states from the `from frame` to the common node
        let (mut pos_fwrd, mut vel_fwrd, mut acc_fwrd, mut frame_fwrd) =
            if from_frame.ephem_origin_hash_match(common_node) {
                (
                    Vector3::zeros(),
                    Vector3::zeros(),
                    Vector3::zeros(),
                    from_frame,
                )
            } else {
                self.translate_to_parent(from_frame, epoch, ab_corr, distance_unit, time_unit)?
            };

        // The bwrd variables are the states from the `to frame` back to the common node
        let (mut pos_bwrd, mut vel_bwrd, mut acc_bwrd, mut frame_bwrd) =
            if to_frame.ephem_origin_hash_match(common_node) {
                (
                    Vector3::zeros(),
                    Vector3::zeros(),
                    Vector3::zeros(),
                    to_frame,
                )
            } else {
                self.translate_to_parent(to_frame, epoch, ab_corr, distance_unit, time_unit)?
            };

        for cur_node_hash in path.iter().take(node_count) {
            if !frame_fwrd.ephem_origin_hash_match(common_node) {
                let (cur_pos_fwrd, cur_vel_fwrd, cur_acc_fwrd, cur_frame_fwrd) =
                    self.translate_to_parent(frame_fwrd, epoch, ab_corr, distance_unit, time_unit)?;

                pos_fwrd += cur_pos_fwrd;
                vel_fwrd += cur_vel_fwrd;
                acc_fwrd += cur_acc_fwrd;
                frame_fwrd = cur_frame_fwrd;
            }

            if !frame_bwrd.ephem_origin_hash_match(common_node) {
                let (cur_pos_bwrd, cur_vel_bwrd, cur_acc_bwrd, cur_frame_bwrd) =
                    self.translate_to_parent(frame_bwrd, epoch, ab_corr, distance_unit, time_unit)?;

                pos_bwrd += cur_pos_bwrd;
                vel_bwrd += cur_vel_bwrd;
                acc_bwrd += cur_acc_bwrd;
                frame_bwrd = cur_frame_bwrd;
            }

            // We know this exist, so we can safely unwrap it
            if cur_node_hash.unwrap() == common_node {
                break;
            }
        }

        Ok((
            pos_fwrd - pos_bwrd,
            vel_fwrd - vel_bwrd,
            acc_fwrd - acc_bwrd,
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

    /// Returns the position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in m, the velocity in m/s, and the acceleration in m/s^2.
    pub fn translate_from_to_m_s(
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
            DistanceUnit::Meter,
            TimeUnit::Second,
        )
    }

    /// Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2.
    pub fn translate_from_to_km_s_geometric(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<(Vector3, Vector3, Vector3), AniseError> {
        self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
    }

    /// Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in m, the velocity in m/s, and the acceleration in m/s^2.
    pub fn translate_from_to_m_s_geometric(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<(Vector3, Vector3, Vector3), AniseError> {
        self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            Aberration::None,
            DistanceUnit::Meter,
            TimeUnit::Second,
        )
    }

    /// Try to construct the path from the source frame all the way to the root ephemeris of this context.
    pub fn translate_to_root(
        &self,
        source: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
        distance_unit: DistanceUnit,
        time_unit: TimeUnit,
    ) -> Result<(Vector3, Vector3, Vector3), AniseError> {
        // Build a tree, set a fixed depth to avoid allocations
        let mut prev_ephem_hash = source.ephemeris_hash;

        let mut pos = Vector3::zeros();
        let mut vel = Vector3::zeros();
        let mut acc = Vector3::zeros();

        for _ in 0..MAX_TREE_DEPTH {
            let idx = self.ephemeris_lut.index_for_hash(&prev_ephem_hash)?;
            let parent_ephem = self.try_ephemeris_data(idx.into())?;
            let parent_hash = parent_ephem.parent_ephemeris_hash;

            let (this_pos, this_vel, this_accel, _) =
                self.translate_to_parent(source, epoch, ab_corr, distance_unit, time_unit)?;

            pos += this_pos;
            vel += this_vel;
            acc += this_accel;

            if parent_hash == self.try_find_context_root()? {
                return Ok((pos, vel, acc));
            } else if let Err(e) = self.ephemeris_lut.index_for_hash(&parent_hash) {
                if e == AniseError::ItemNotFound {
                    // We have reached the root of this ephemeris and it has no parent.
                    error!("{parent_hash} has no parent in this context");
                    return Ok((pos, vel, acc));
                }
            }
            prev_ephem_hash = parent_hash;
        }
        Err(AniseError::MaxTreeDepth)
    }

    /// Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
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
