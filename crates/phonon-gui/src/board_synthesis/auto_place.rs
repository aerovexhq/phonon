#![deny(unsafe_code)]

//! Physical Layout Organization & De-clumping Auto-Placement Algorithm.
//!
//! When abstract components explode into multiple physical chips (e.g., an XOR gate
//! decomposing into multiple 7400-series ICs), their initial positions clump together.
//! This module detects package collisions and runs a force-directed relaxation algorithm
//! to push overlapping chips apart onto a clean, fabrication-ready 2.54mm PCB grid.

use crate::board_synthesis::footprint::FootprintInstance;
use egui::{Pos2, Vec2};

/// Parameters governing physical layout auto-placement and de-clumping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoPlacementParams {
    /// Minimum clearance between component package bodies in millimeters (default 3.0mm).
    pub package_clearance_mm: f32,
    /// Standard component placement grid snapping in millimeters (default 2.54mm / 100 mil).
    pub grid_pitch_mm: f32,
    /// Margin between board edge and outer components in millimeters (default 6.0mm).
    pub board_margin_mm: f32,
    /// Maximum relaxation iterations (default 60).
    pub max_iterations: usize,
}

impl Default for AutoPlacementParams {
    fn default() -> Self {
        Self {
            package_clearance_mm: 3.5,
            grid_pitch_mm: 2.54,
            board_margin_mm: 6.0,
            max_iterations: 60,
        }
    }
}

/// Result of auto-placement containing arranged chips and computed board dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedBoardLayout {
    pub chips: Vec<FootprintInstance>,
    pub board_width_mm: f32,
    pub board_height_mm: f32,
    pub overlaps_resolved: usize,
}

/// De-clumping and layout auto-placement engine.
pub struct AutoPlacer;

impl AutoPlacer {
    /// Detects if any two chips have overlapping or colliding bounding boxes.
    pub fn count_collisions(chips: &[FootprintInstance], clearance_mm: f32) -> usize {
        let mut count = 0;
        let n = chips.len();
        for i in 0..n {
            let box_i = chips[i].bounding_box_mm(clearance_mm);
            for j in (i + 1)..n {
                let box_j = chips[j].bounding_box_mm(clearance_mm);
                if box_i.intersects(box_j) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Organizes and declumps chips, pushing overlapping packages apart and snapping to grid.
    pub fn declump_and_place(
        mut chips: Vec<FootprintInstance>,
        params: &AutoPlacementParams,
    ) -> PlacedBoardLayout {
        if chips.is_empty() {
            return PlacedBoardLayout {
                chips,
                board_width_mm: 50.0,
                board_height_mm: 40.0,
                overlaps_resolved: 0,
            };
        }

        let initial_collisions = Self::count_collisions(&chips, params.package_clearance_mm);
        let n = chips.len();

        // If multiple chips are placed at the exact same location (e.g. from an exploded gate),
        // give each an initial distinct directional offset to break symmetry.
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = (chips[i].center_mm - chips[j].center_mm).length();
                if dist < 0.1 {
                    let offset_x = ((j as f32) * 22.0) * params.grid_pitch_mm;
                    chips[j].center_mm.x += offset_x;
                }
            }
        }

        // Iterative Force-Directed Relaxation Loop
        for _iter in 0..params.max_iterations {
            let mut any_movement = false;
            let mut displacements = vec![Vec2::ZERO; n];

            for i in 0..n {
                let box_i = chips[i].bounding_box_mm(params.package_clearance_mm);
                for j in (i + 1)..n {
                    let box_j = chips[j].bounding_box_mm(params.package_clearance_mm);

                    if box_i.intersects(box_j) {
                        any_movement = true;
                        let delta = chips[i].center_mm - chips[j].center_mm;
                        let dist = delta.length();

                        let dir = if dist < 1e-3 {
                            Vec2::new(1.0, 0.0)
                        } else {
                            delta / dist
                        };

                        // Compute required separation based on dimensions
                        let req_dist = (box_i.size().length() + box_j.size().length()) * 0.45;
                        let overlap = (req_dist - dist).max(1.0);
                        let push = dir * (overlap * 0.55);

                        displacements[i] += push;
                        displacements[j] -= push;
                    }
                }
            }

            for i in 0..n {
                chips[i].center_mm += displacements[i];
            }

            if !any_movement {
                break;
            }
        }

        // Snap all chips to standard PCB component placement grid
        for chip in &mut chips {
            let gx = (chip.center_mm.x / params.grid_pitch_mm).round() * params.grid_pitch_mm;
            let gy = (chip.center_mm.y / params.grid_pitch_mm).round() * params.grid_pitch_mm;
            chip.center_mm = Pos2::new(gx, gy);
        }

        // Calculate minimum board dimensions enclosing all packages with margins
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for chip in &chips {
            let b = chip.bounding_box_mm(params.package_clearance_mm);
            min_x = min_x.min(b.min.x);
            min_y = min_y.min(b.min.y);
            max_x = max_x.max(b.max.x);
            max_y = max_y.max(b.max.y);
        }

        // Shift so min bounds are at board_margin_mm
        let shift = Vec2::new(params.board_margin_mm - min_x, params.board_margin_mm - min_y);
        for chip in &mut chips {
            chip.center_mm += shift;
            // Re-snap to grid after shift
            let gx = (chip.center_mm.x / params.grid_pitch_mm).round() * params.grid_pitch_mm;
            let gy = (chip.center_mm.y / params.grid_pitch_mm).round() * params.grid_pitch_mm;
            chip.center_mm = Pos2::new(gx, gy);
        }

        let total_w = (max_x - min_x + 2.0 * params.board_margin_mm).max(40.0);
        let total_h = (max_y - min_y + 2.0 * params.board_margin_mm).max(30.0);

        PlacedBoardLayout {
            chips,
            board_width_mm: (total_w / params.grid_pitch_mm).ceil() * params.grid_pitch_mm,
            board_height_mm: (total_h / params.grid_pitch_mm).ceil() * params.grid_pitch_mm,
            overlaps_resolved: initial_collisions,
        }
    }
}
