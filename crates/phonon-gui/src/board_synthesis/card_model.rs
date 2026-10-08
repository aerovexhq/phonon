#![deny(unsafe_code)]

//! Complete 3D Synthesized Physical Board Card Data Model.
//!
//! Encapsulates physical substrate geometry, layer stackup, component packages,
//! routed copper traces, and 3D geometric mesh primitives for interactive CAD viewing.

use crate::board_synthesis::footprint::FootprintInstance;
use crate::board_synthesis::parasitics::PhysicalWiringRealismReport;
use crate::board_synthesis::trace_router::RoutedPhysicalNet;
use egui::Color32;

/// Visual solder mask color options for the physical PCB card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolderMaskColor {
    ObsidianMatteBlack,
    EmeraldGreen,
    DeepNavyBlue,
    RubyRed,
    CleanWhite,
}

impl SolderMaskColor {
    pub fn substrate_color(&self) -> Color32 {
        match self {
            Self::ObsidianMatteBlack => Color32::from_rgb(26, 28, 32),
            Self::EmeraldGreen => Color32::from_rgb(18, 75, 42),
            Self::DeepNavyBlue => Color32::from_rgb(18, 38, 75),
            Self::RubyRed => Color32::from_rgb(85, 20, 24),
            Self::CleanWhite => Color32::from_rgb(225, 228, 235),
        }
    }

    pub fn edge_color(&self) -> Color32 {
        match self {
            Self::ObsidianMatteBlack => Color32::from_rgb(45, 48, 55),
            Self::EmeraldGreen => Color32::from_rgb(28, 105, 58),
            Self::DeepNavyBlue => Color32::from_rgb(28, 55, 105),
            Self::RubyRed => Color32::from_rgb(115, 32, 38),
            Self::CleanWhite => Color32::from_rgb(190, 195, 205),
        }
    }
}

/// A 3D vertex with spatial coordinates (X, Y, Z) in board millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// A planar 3D polygon face for software 3D rendering and depth-sorting.
#[derive(Debug, Clone, PartialEq)]
pub struct Face3D {
    pub vertices: Vec<Vertex3D>,
    pub color: Color32,
    /// Center point in 3D for depth-sorting.
    pub center: Vertex3D,
    /// Outward surface normal vector.
    pub normal: Vertex3D,
}

/// Synthesized 3D Physical Board Card assembly.
#[derive(Debug, Clone, PartialEq)]
pub struct SynthesizedPhysicalCard {
    pub board_width_mm: f32,
    pub board_height_mm: f32,
    pub board_thickness_mm: f32,
    pub solder_mask: SolderMaskColor,
    pub chips: Vec<FootprintInstance>,
    pub routed_nets: Vec<RoutedPhysicalNet>,
    pub wiring_report: PhysicalWiringRealismReport,
}

impl SynthesizedPhysicalCard {
    pub fn new(
        board_width_mm: f32,
        board_height_mm: f32,
        chips: Vec<FootprintInstance>,
        routed_nets: Vec<RoutedPhysicalNet>,
        wiring_report: PhysicalWiringRealismReport,
    ) -> Self {
        Self {
            board_width_mm,
            board_height_mm,
            board_thickness_mm: 1.6, // Standard 1.6mm FR4 core
            solder_mask: SolderMaskColor::ObsidianMatteBlack,
            chips,
            routed_nets,
            wiring_report,
        }
    }

    /// Generates 3D polygonal faces for the PCB substrate and all mounted components.
    pub fn generate_3d_faces(&self) -> Vec<Face3D> {
        let mut faces = Vec::new();
        let hw = self.board_width_mm / 2.0;
        let hh = self.board_height_mm / 2.0;
        let t = self.board_thickness_mm;

        // 1. PCB Top Face (Z = 0)
        let top_color = self.solder_mask.substrate_color();
        faces.push(Face3D {
            vertices: vec![
                Vertex3D::new(-hw, -hh, 0.0),
                Vertex3D::new(hw, -hh, 0.0),
                Vertex3D::new(hw, hh, 0.0),
                Vertex3D::new(-hw, hh, 0.0),
            ],
            color: top_color,
            center: Vertex3D::new(0.0, 0.0, 0.0),
            normal: Vertex3D::new(0.0, 0.0, 1.0),
        });

        // 2. PCB Bottom Face (Z = -t)
        let bot_color = Color32::from_rgb(
            (top_color.r() as f32 * 0.75) as u8,
            (top_color.g() as f32 * 0.75) as u8,
            (top_color.b() as f32 * 0.75) as u8,
        );
        faces.push(Face3D {
            vertices: vec![
                Vertex3D::new(-hw, -hh, -t),
                Vertex3D::new(-hw, hh, -t),
                Vertex3D::new(hw, hh, -t),
                Vertex3D::new(hw, -hh, -t),
            ],
            color: bot_color,
            center: Vertex3D::new(0.0, 0.0, -t),
            normal: Vertex3D::new(0.0, 0.0, -1.0),
        });

        // 3. Four Extruded Board Edges
        let edge_color = self.solder_mask.edge_color();
        // Front Edge (Y = -hh)
        faces.push(Face3D {
            vertices: vec![
                Vertex3D::new(-hw, -hh, 0.0),
                Vertex3D::new(hw, -hh, 0.0),
                Vertex3D::new(hw, -hh, -t),
                Vertex3D::new(-hw, -hh, -t),
            ],
            color: edge_color,
            center: Vertex3D::new(0.0, -hh, -t / 2.0),
            normal: Vertex3D::new(0.0, -1.0, 0.0),
        });
        // Back Edge (Y = +hh)
        faces.push(Face3D {
            vertices: vec![
                Vertex3D::new(-hw, hh, 0.0),
                Vertex3D::new(-hw, hh, -t),
                Vertex3D::new(hw, hh, -t),
                Vertex3D::new(hw, hh, 0.0),
            ],
            color: edge_color,
            center: Vertex3D::new(0.0, hh, -t / 2.0),
            normal: Vertex3D::new(0.0, 1.0, 0.0),
        });
        // Left Edge (X = -hw)
        faces.push(Face3D {
            vertices: vec![
                Vertex3D::new(-hw, -hh, 0.0),
                Vertex3D::new(-hw, -hh, -t),
                Vertex3D::new(-hw, hh, -t),
                Vertex3D::new(-hw, hh, 0.0),
            ],
            color: edge_color,
            center: Vertex3D::new(-hw, 0.0, -t / 2.0),
            normal: Vertex3D::new(-1.0, 0.0, 0.0),
        });
        // Right Edge (X = +hw)
        faces.push(Face3D {
            vertices: vec![
                Vertex3D::new(hw, -hh, 0.0),
                Vertex3D::new(hw, hh, 0.0),
                Vertex3D::new(hw, hh, -t),
                Vertex3D::new(hw, -hh, -t),
            ],
            color: edge_color,
            center: Vertex3D::new(hw, 0.0, -t / 2.0),
            normal: Vertex3D::new(1.0, 0.0, 0.0),
        });

        // 4. Mounted Physical Component IC Packages
        for chip in &self.chips {
            let cx = chip.center_mm.x - hw;
            let cy = chip.center_mm.y - hh;
            let dims = chip.package_type.body_dimensions_mm();
            let cw = dims.x / 2.0;
            let ch = dims.y / 2.0;
            let cz = chip.package_type.height_mm();

            let epoxy_top = Color32::from_rgb(42, 44, 48);
            let epoxy_side = Color32::from_rgb(30, 32, 35);
            let silver_pin = Color32::from_rgb(195, 205, 215);

            // Chip Top Face
            faces.push(Face3D {
                vertices: vec![
                    Vertex3D::new(cx - cw, cy - ch, cz),
                    Vertex3D::new(cx + cw, cy - ch, cz),
                    Vertex3D::new(cx + cw, cy + ch, cz),
                    Vertex3D::new(cx - cw, cy + ch, cz),
                ],
                color: epoxy_top,
                center: Vertex3D::new(cx, cy, cz),
                normal: Vertex3D::new(0.0, 0.0, 1.0),
            });

            // Chip 4 Sides
            faces.push(Face3D {
                vertices: vec![
                    Vertex3D::new(cx - cw, cy - ch, 0.0),
                    Vertex3D::new(cx + cw, cy - ch, 0.0),
                    Vertex3D::new(cx + cw, cy - ch, cz),
                    Vertex3D::new(cx - cw, cy - ch, cz),
                ],
                color: epoxy_side,
                center: Vertex3D::new(cx, cy - ch, cz / 2.0),
                normal: Vertex3D::new(0.0, -1.0, 0.0),
            });
            faces.push(Face3D {
                vertices: vec![
                    Vertex3D::new(cx - cw, cy + ch, 0.0),
                    Vertex3D::new(cx - cw, cy + ch, cz),
                    Vertex3D::new(cx + cw, cy + ch, cz),
                    Vertex3D::new(cx + cw, cy + ch, 0.0),
                ],
                color: epoxy_side,
                center: Vertex3D::new(cx, cy + ch, cz / 2.0),
                normal: Vertex3D::new(0.0, 1.0, 0.0),
            });
            faces.push(Face3D {
                vertices: vec![
                    Vertex3D::new(cx - cw, cy - ch, 0.0),
                    Vertex3D::new(cx - cw, cy - ch, cz),
                    Vertex3D::new(cx - cw, cy + ch, cz),
                    Vertex3D::new(cx - cw, cy + ch, 0.0),
                ],
                color: epoxy_side,
                center: Vertex3D::new(cx - cw, cy, cz / 2.0),
                normal: Vertex3D::new(-1.0, 0.0, 0.0),
            });
            faces.push(Face3D {
                vertices: vec![
                    Vertex3D::new(cx + cw, cy - ch, 0.0),
                    Vertex3D::new(cx + cw, cy + ch, 0.0),
                    Vertex3D::new(cx + cw, cy + ch, cz),
                    Vertex3D::new(cx + cw, cy - ch, cz),
                ],
                color: epoxy_side,
                center: Vertex3D::new(cx + cw, cy, cz / 2.0),
                normal: Vertex3D::new(1.0, 0.0, 0.0),
            });

            // Physical metallic through-hole pads and pins
            for pad in &chip.pads {
                let px = cx + pad.rel_pos.x;
                let py = cy + pad.rel_pos.y;
                let pr = pad.pad_diameter_mm / 2.0;

                // Gold solder pad on board surface
                faces.push(Face3D {
                    vertices: vec![
                        Vertex3D::new(px - pr, py - pr, 0.02),
                        Vertex3D::new(px + pr, py - pr, 0.02),
                        Vertex3D::new(px + pr, py + pr, 0.02),
                        Vertex3D::new(px - pr, py + pr, 0.02),
                    ],
                    color: Color32::from_rgb(220, 180, 75), // Gold plated pad
                    center: Vertex3D::new(px, py, 0.02),
                    normal: Vertex3D::new(0.0, 0.0, 1.0),
                });

                // Silver IC pin leg
                if pad.is_through_hole {
                    faces.push(Face3D {
                        vertices: vec![
                            Vertex3D::new(px - 0.25, py - 0.25, 0.0),
                            Vertex3D::new(px + 0.25, py - 0.25, 0.0),
                            Vertex3D::new(px + 0.25, py + 0.25, cz * 0.7),
                            Vertex3D::new(px - 0.25, py + 0.25, cz * 0.7),
                        ],
                        color: silver_pin,
                        center: Vertex3D::new(px, py, cz * 0.35),
                        normal: Vertex3D::new(0.0, 1.0, 0.0),
                    });
                }
            }
        }

        faces
    }
}
