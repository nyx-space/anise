/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::event::{EventArc, EventEdge};
use hifitime::Epoch;
use std::cmp::Ordering;

#[cfg(feature = "python")]
use pyo3::pyfunction;

#[derive(Debug, PartialEq, Eq)]
struct EventPoint {
    epoch: Epoch,
    kind: EventEdge,
    list_id: usize, // Which timeline (0, 1, 2...) this point belongs to
}

// Custom sorting for EventPoint
// 1. Sort by epoch
// 2. If epochs are equal, process START before END
//    (This correctly handles "touching" intervals like (10, 20) and (20, 30))
impl Ord for EventPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.epoch.cmp(&other.epoch).then_with(|| {
            // Process rise before fall
            self.kind.cmp(&other.kind)
        })
    }
}

impl PartialOrd for EventPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Finds the intersection of multiple event arc timelines.
///
/// Input: A Vec where each element is a timeline (Vec<EventArc>)
///        e.g., [ timeline_A_arcs, timeline_B_arcs, ... ]
///
/// Output: A Vec of (Epoch, Epoch) windows where *all* timelines were active.

#[cfg_attr(feature = "python", pyfunction)]
pub fn find_arc_intersections(timelines: Vec<Vec<EventArc>>) -> Vec<(Epoch, Epoch)> {
    let num_timelines = timelines.len();
    if num_timelines == 0 {
        return Vec::new();
    }

    let mut all_points: Vec<EventPoint> = Vec::new();

    // 1. Flatten all timelines into a single list of EventPoints
    //    We tag each point with its timeline's ID (its index)
    for (list_id, arcs) in timelines.iter().enumerate() {
        for arc in arcs {
            all_points.push(EventPoint {
                epoch: arc.rise.orbit.epoch, // Use the epoch from the EventDetails
                kind: EventEdge::Rising,
                list_id,
            });
            all_points.push(EventPoint {
                epoch: arc.fall.orbit.epoch,
                kind: EventEdge::Falling,
                list_id,
            });
        }
    }

    // 2. Sort all points chronologically
    all_points.sort();

    // 3. Sweep the line
    let mut result_windows: Vec<(Epoch, Epoch)> = Vec::new();
    let mut intersection_start: Option<Epoch> = None;

    // This array tracks the "active" state for each timeline.
    // e.g., active_state[0] = true means timeline 0 is in an arc.
    let mut active_state: Vec<bool> = vec![false; num_timelines];

    for point in all_points {
        let current_epoch = point.epoch;

        // Check the state *before* we apply the change
        let was_fully_intersecting = active_state.iter().all(|&is_active| is_active);

        // Update the state for the timeline this point belongs to
        match point.kind {
            EventEdge::Rising => active_state[point.list_id] = true,
            EventEdge::Falling => active_state[point.list_id] = false,
            _ => unreachable!(),
        }

        // Check the state *after* the change
        let is_fully_intersecting = active_state.iter().all(|&is_active| is_active);

        // Now, compare the before/after states to find boundaries
        if !was_fully_intersecting && is_fully_intersecting {
            // We just entered a full intersection.
            // This epoch is the start of the window.
            intersection_start = Some(current_epoch);
        } else if was_fully_intersecting && !is_fully_intersecting {
            // We just left a full intersection.
            // This epoch is the end of the window.
            if let Some(start_epoch) = intersection_start.take() {
                // Add the window, but only if it's valid (start < end)
                if current_epoch > start_epoch {
                    result_windows.push((start_epoch, current_epoch));
                }
            }
        }
        // If state didn't change (e.g., `false` -> `false` or `true` -> `true`),
        // we do nothing, which is correct.
    }

    result_windows
}
