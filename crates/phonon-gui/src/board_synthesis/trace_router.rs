#![deny(unsafe_code)]

//! Physical PCB Copper Trace Organization & Routing Algorithm.
//!
//! Connects physical component pads across dual-layer (Top / Bottom) copper
//! with orthogonal Manhattan routing, layer assignments, and geometric length
//! tracking for downstream parasitic RLC and electromagnetic coupling extraction.

use crate::board_synthesis::footprint::FootprintInstance;
use crate::board_synthesis::tech_mapping::PhysicalNetConnection;
use egui::Pos2;
use std::collections::HashMap;

/// PCB copper layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CopperLayer {
    /// Top copper layer (predominantly horizontal routing, rendered in warm copper/red).
    Top,
    /// Bottom copper layer (predominantly vertical routing, rendered in cool blue/teal).
    Bottom,
}

/// A physical straight segment of copper trace on a specific PCB layer.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalTraceSegment {
    pub start_mm: Pos2,
    pub end_mm: Pos2,
    pub layer: CopperLayer,
    pub width_mm: f32,
}

impl PhysicalTraceSegment {
    pub fn length_mm(&self) -> f32 {
        (self.end_mm - self.start_mm).length()
    }
}

/// A routed physical net on the PCB card containing one or more trace segments.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutedPhysicalNet {
    pub net_name: String,
    pub segments: Vec<PhysicalTraceSegment>,
    pub is_power: bool,
    pub total_length_mm: f32,
}

/// Physical board router engine.
pub struct BoardAutoRouter;

impl BoardAutoRouter {
    /// Routes copper traces connecting all physical pads sharing the same net.
    pub fn route_nets(
        chips: &[FootprintInstance],
        connections: &[PhysicalNetConnection],
    ) -> Vec<RoutedPhysicalNet> {
        // 1. Group pad locations by net name
        let mut net_pads: HashMap<String, (Vec<Pos2>, bool)> = HashMap::new();

        // Index chips by ID for fast pad coordinate lookup
        let chip_map: HashMap<usize, &FootprintInstance> =
            chips.iter().map(|c| (c.id, c)).collect();

        for conn in connections {
            if let Some(chip) = chip_map.get(&conn.chip_id) {
                if let Some(pos) = chip.pad_world_pos(conn.pin_number) {
                    let entry = net_pads
                        .entry(conn.net_name.clone())
                        .or_insert_with(|| (Vec::new(), conn.is_power_or_gnd));
                    entry.0.push(pos);
                    if conn.is_power_or_gnd {
                        entry.1 = true;
                    }
                }
            }
        }

        let mut routed_nets = Vec::new();

        // 2. Route each net with Manhattan orthogonal segments
        for (net_name, (pads, is_power)) in net_pads {
            if pads.len() < 2 {
                continue;
            }

            let trace_width = if is_power { 0.50 } else { 0.25 }; // mm
            let mut segments = Vec::new();
            let mut total_length = 0.0f32;

            // Connect pads sequentially (Minimum Spanning Tree chain approximation)
            for i in 0..(pads.len() - 1) {
                let p1 = pads[i];
                let p2 = pads[i + 1];

                // If strictly aligned horizontally or vertically
                if (p1.y - p2.y).abs() < 1e-3 {
                    let seg = PhysicalTraceSegment {
                        start_mm: p1,
                        end_mm: p2,
                        layer: CopperLayer::Top,
                        width_mm: trace_width,
                    };
                    total_length += seg.length_mm();
                    segments.push(seg);
                } else if (p1.x - p2.x).abs() < 1e-3 {
                    let seg = PhysicalTraceSegment {
                        start_mm: p1,
                        end_mm: p2,
                        layer: CopperLayer::Bottom,
                        width_mm: trace_width,
                    };
                    total_length += seg.length_mm();
                    segments.push(seg);
                } else {
                    // Orthogonal dogleg: Horizontal on Top Layer, Vertical on Bottom Layer
                    let corner = Pos2::new(p2.x, p1.y);
                    let seg1 = PhysicalTraceSegment {
                        start_mm: p1,
                        end_mm: corner,
                        layer: CopperLayer::Top,
                        width_mm: trace_width,
                    };
                    let seg2 = PhysicalTraceSegment {
                        start_mm: corner,
                        end_mm: p2,
                        layer: CopperLayer::Bottom,
                        width_mm: trace_width,
                    };
                    total_length += seg1.length_mm() + seg2.length_mm();
                    segments.push(seg1);
                    segments.push(seg2);
                }
            }

            routed_nets.push(RoutedPhysicalNet {
                net_name,
                segments,
                is_power,
                total_length_mm: total_length,
            });
        }

        // Sort for deterministic test output
        routed_nets.sort_by(|a, b| a.net_name.cmp(&b.net_name));
        routed_nets
    }
}
