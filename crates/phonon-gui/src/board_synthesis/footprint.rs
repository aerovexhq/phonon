#![deny(unsafe_code)]

//! Physical Component Footprints and IC Packaging Definitions.
//!
//! Provides geometric specifications, through-hole pin mappings, and physical
//! dimensions for realistic synthesized printed circuit board (PCB) assembly.

use egui::{Pos2, Rect, Vec2};

/// Industry standard physical IC package types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageType {
    /// 14-pin Dual In-line Package (standard 7400-series logic ICs).
    Dip14,
    /// 8-pin Dual In-line Package (standard 555 timer, op-amps).
    Dip8,
    /// 16-pin Dual In-line Package (shift registers, counters).
    Dip16,
    /// Surface Mount Device 0805 metric 2012 (passives).
    Smd0805,
    /// TO-92 3-terminal discrete semiconductor package (transistors, small diodes).
    To92,
    /// 2-pin 2.54mm pitch through-hole header (power, single port).
    PinHeader2,
    /// 4-pin 2.54mm pitch through-hole header.
    PinHeader4,
}

impl PackageType {
    /// Physical body dimensions (length_x, width_y) in millimeters.
    pub fn body_dimensions_mm(&self) -> Vec2 {
        match self {
            Self::Dip14 => Vec2::new(19.2, 6.4),
            Self::Dip8 => Vec2::new(9.6, 6.4),
            Self::Dip16 => Vec2::new(20.0, 6.4),
            Self::Smd0805 => Vec2::new(2.0, 1.25),
            Self::To92 => Vec2::new(4.5, 3.8),
            Self::PinHeader2 => Vec2::new(5.08, 2.54),
            Self::PinHeader4 => Vec2::new(10.16, 2.54),
        }
    }

    /// Package 3D height in millimeters above the board surface.
    pub fn height_mm(&self) -> f32 {
        match self {
            Self::Dip14 | Self::Dip8 | Self::Dip16 => 3.5,
            Self::Smd0805 => 0.6,
            Self::To92 => 4.8,
            Self::PinHeader2 | Self::PinHeader4 => 8.5,
        }
    }

    /// Returns the complete list of physical pin pads with their relative coordinates in mm.
    pub fn generate_pads(&self) -> Vec<PhysicalPad> {
        match self {
            Self::Dip14 => {
                let mut pads = Vec::with_capacity(14);
                let row_spacing = 7.62; // 300 mil standard row separation
                let pitch = 2.54;       // 100 mil pin pitch
                let start_x = -3.0 * pitch;

                // Pins 1 to 7: bottom row (Y = +row_spacing / 2)
                for i in 0..7 {
                    let pin_num = i + 1;
                    let x = start_x + (i as f32) * pitch;
                    pads.push(PhysicalPad {
                        pin_number: pin_num,
                        pin_name: format!("P{}", pin_num),
                        rel_pos: Pos2::new(x, row_spacing / 2.0),
                        is_through_hole: true,
                        pad_diameter_mm: 1.6,
                        drill_diameter_mm: 0.8,
                    });
                }
                // Pins 8 to 14: top row (Y = -row_spacing / 2), numbered right-to-left
                for i in 0..7 {
                    let pin_num = 8 + i;
                    let x = -start_x - (i as f32) * pitch;
                    pads.push(PhysicalPad {
                        pin_number: pin_num,
                        pin_name: format!("P{}", pin_num),
                        rel_pos: Pos2::new(x, -row_spacing / 2.0),
                        is_through_hole: true,
                        pad_diameter_mm: 1.6,
                        drill_diameter_mm: 0.8,
                    });
                }
                pads
            }
            Self::Dip8 => {
                let mut pads = Vec::with_capacity(8);
                let row_spacing = 7.62;
                let pitch = 2.54;
                let start_x = -1.5 * pitch;

                for i in 0..4 {
                    let pin_num = i + 1;
                    let x = start_x + (i as f32) * pitch;
                    pads.push(PhysicalPad {
                        pin_number: pin_num,
                        pin_name: format!("P{}", pin_num),
                        rel_pos: Pos2::new(x, row_spacing / 2.0),
                        is_through_hole: true,
                        pad_diameter_mm: 1.6,
                        drill_diameter_mm: 0.8,
                    });
                }
                for i in 0..4 {
                    let pin_num = 5 + i;
                    let x = -start_x - (i as f32) * pitch;
                    pads.push(PhysicalPad {
                        pin_number: pin_num,
                        pin_name: format!("P{}", pin_num),
                        rel_pos: Pos2::new(x, -row_spacing / 2.0),
                        is_through_hole: true,
                        pad_diameter_mm: 1.6,
                        drill_diameter_mm: 0.8,
                    });
                }
                pads
            }
            Self::Dip16 => {
                let mut pads = Vec::with_capacity(16);
                let row_spacing = 7.62;
                let pitch = 2.54;
                let start_x = -3.5 * pitch;

                for i in 0..8 {
                    let pin_num = i + 1;
                    let x = start_x + (i as f32) * pitch;
                    pads.push(PhysicalPad {
                        pin_number: pin_num,
                        pin_name: format!("P{}", pin_num),
                        rel_pos: Pos2::new(x, row_spacing / 2.0),
                        is_through_hole: true,
                        pad_diameter_mm: 1.6,
                        drill_diameter_mm: 0.8,
                    });
                }
                for i in 0..8 {
                    let pin_num = 9 + i;
                    let x = -start_x - (i as f32) * pitch;
                    pads.push(PhysicalPad {
                        pin_number: pin_num,
                        pin_name: format!("P{}", pin_num),
                        rel_pos: Pos2::new(x, -row_spacing / 2.0),
                        is_through_hole: true,
                        pad_diameter_mm: 1.6,
                        drill_diameter_mm: 0.8,
                    });
                }
                pads
            }
            Self::Smd0805 => vec![
                PhysicalPad {
                    pin_number: 1,
                    pin_name: "1".to_string(),
                    rel_pos: Pos2::new(-0.9, 0.0),
                    is_through_hole: false,
                    pad_diameter_mm: 1.0,
                    drill_diameter_mm: 0.0,
                },
                PhysicalPad {
                    pin_number: 2,
                    pin_name: "2".to_string(),
                    rel_pos: Pos2::new(0.9, 0.0),
                    is_through_hole: false,
                    pad_diameter_mm: 1.0,
                    drill_diameter_mm: 0.0,
                },
            ],
            Self::To92 => vec![
                PhysicalPad {
                    pin_number: 1,
                    pin_name: "1".to_string(),
                    rel_pos: Pos2::new(-1.27, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.4,
                    drill_diameter_mm: 0.7,
                },
                PhysicalPad {
                    pin_number: 2,
                    pin_name: "2".to_string(),
                    rel_pos: Pos2::new(0.0, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.4,
                    drill_diameter_mm: 0.7,
                },
                PhysicalPad {
                    pin_number: 3,
                    pin_name: "3".to_string(),
                    rel_pos: Pos2::new(1.27, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.4,
                    drill_diameter_mm: 0.7,
                },
            ],
            Self::PinHeader2 => vec![
                PhysicalPad {
                    pin_number: 1,
                    pin_name: "1".to_string(),
                    rel_pos: Pos2::new(-1.27, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.7,
                    drill_diameter_mm: 1.0,
                },
                PhysicalPad {
                    pin_number: 2,
                    pin_name: "2".to_string(),
                    rel_pos: Pos2::new(1.27, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.7,
                    drill_diameter_mm: 1.0,
                },
            ],
            Self::PinHeader4 => vec![
                PhysicalPad {
                    pin_number: 1,
                    pin_name: "1".to_string(),
                    rel_pos: Pos2::new(-3.81, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.7,
                    drill_diameter_mm: 1.0,
                },
                PhysicalPad {
                    pin_number: 2,
                    pin_name: "2".to_string(),
                    rel_pos: Pos2::new(-1.27, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.7,
                    drill_diameter_mm: 1.0,
                },
                PhysicalPad {
                    pin_number: 3,
                    pin_name: "3".to_string(),
                    rel_pos: Pos2::new(1.27, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.7,
                    drill_diameter_mm: 1.0,
                },
                PhysicalPad {
                    pin_number: 4,
                    pin_name: "4".to_string(),
                    rel_pos: Pos2::new(3.81, 0.0),
                    is_through_hole: true,
                    pad_diameter_mm: 1.7,
                    drill_diameter_mm: 1.0,
                },
            ],
        }
    }
}

/// A physical contact pad or through-hole drill point on the PCB.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalPad {
    pub pin_number: usize,
    pub pin_name: String,
    /// Offset in millimeters relative to the component package center.
    pub rel_pos: Pos2,
    pub is_through_hole: bool,
    pub pad_diameter_mm: f32,
    pub drill_diameter_mm: f32,
}

/// A physical footprint instance with world coordinates on the board.
#[derive(Debug, Clone, PartialEq)]
pub struct FootprintInstance {
    pub id: usize,
    pub designator: String,
    pub chip_part_number: String,
    pub package_type: PackageType,
    /// Board coordinate in millimeters (origin at board center or bottom-left).
    pub center_mm: Pos2,
    /// Rotation in degrees (0, 90, 180, 270).
    pub rotation_deg: f32,
    /// Assigned physical pads.
    pub pads: Vec<PhysicalPad>,
}

impl FootprintInstance {
    pub fn new(
        id: usize,
        designator: impl Into<String>,
        chip_part_number: impl Into<String>,
        package_type: PackageType,
        center_mm: Pos2,
    ) -> Self {
        let pads = package_type.generate_pads();
        Self {
            id,
            designator: designator.into(),
            chip_part_number: chip_part_number.into(),
            package_type,
            center_mm,
            rotation_deg: 0.0,
            pads,
        }
    }

    /// Computes the bounding box of the physical package in board coordinates (mm).
    pub fn bounding_box_mm(&self, clearance_mm: f32) -> Rect {
        let dims = self.package_type.body_dimensions_mm();
        let (w, h) = if (self.rotation_deg / 90.0).round() as i32 % 2 == 1 {
            (dims.y, dims.x)
        } else {
            (dims.x, dims.y)
        };
        let half = Vec2::new(w / 2.0 + clearance_mm, h / 2.0 + clearance_mm);
        Rect::from_min_max(self.center_mm - half, self.center_mm + half)
    }

    /// Computes the absolute board coordinates of a pad by pin number.
    pub fn pad_world_pos(&self, pin_number: usize) -> Option<Pos2> {
        let pad = self.pads.iter().find(|p| p.pin_number == pin_number)?;
        let rad = self.rotation_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();
        let rx = pad.rel_pos.x * cos_a - pad.rel_pos.y * sin_a;
        let ry = pad.rel_pos.x * sin_a + pad.rel_pos.y * cos_a;
        Some(Pos2::new(self.center_mm.x + rx, self.center_mm.y + ry))
    }
}
