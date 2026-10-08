#![deny(unsafe_code)]

//! Technology Mapping and Abstract Gate Explosion Engine.
//!
//! Translates high-level behavioral schematic symbols into concrete industry-standard
//! logic IC packages (such as 7400-series CMOS/TTL chips), enforces physical synthesis
//! validation constraints, and assigns logical net connectivity to physical pin pads.

use crate::board_synthesis::footprint::{FootprintInstance, PackageType};
use crate::schematic::components::{ComponentKind, SchematicComponent};
use crate::schematic::wire::SchematicWire;
use egui::Pos2;
use std::collections::HashMap;
use thiserror::Error;

/// Errors produced during physical board card synthesis.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum SynthesisError {
    #[error("Physical synthesis requires concrete chips. Component '{designator}' ({kind:?}) is an abstract schematic gate without a physical footprint. Enable 'Technology Mapping: Explode Abstract Gates' or assign a concrete IC package.")]
    AbstractGateNotExploded {
        designator: String,
        kind: ComponentKind,
    },
    #[error("Component '{designator}' has no valid physical packaging model.")]
    MissingFootprint { designator: String },
    #[error("Empty schematic: at least one component is required to synthesize a physical card.")]
    EmptySchematic,
}

/// Technology mapping options for physical board synthesis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TechMappingOptions {
    /// When true, automatically explodes/maps abstract logic gates to real 7400-series physical ICs.
    /// When false, synthesis fails if any abstract gates are present.
    pub auto_explode_gates: bool,
    /// When true, decomposes XOR and complex gates strictly into universal NAND gate ICs (74HC00).
    pub force_universal_nand_explosion: bool,
    /// Clearance distance in millimeters for package placement.
    pub component_clearance_mm: f32,
}

impl Default for TechMappingOptions {
    fn default() -> Self {
        Self {
            auto_explode_gates: true,
            force_universal_nand_explosion: false,
            component_clearance_mm: 2.54,
        }
    }
}

/// A physical net connection on the synthesized board.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalNetConnection {
    pub net_name: String,
    pub chip_id: usize,
    pub pin_number: usize,
    pub is_power_or_gnd: bool,
}

/// The synthesized logical-to-physical mapping report.
#[derive(Debug, Clone)]
pub struct MappedPhysicalBoard {
    pub chips: Vec<FootprintInstance>,
    pub net_connections: Vec<PhysicalNetConnection>,
    pub exploded_count: usize,
}

/// Technology Mapping Engine.
pub struct TechMapper;

impl TechMapper {
    /// Evaluates whether a schematic component is an abstract ideal gate needing explosion.
    pub fn is_abstract_gate(kind: ComponentKind) -> bool {
        matches!(
            kind,
            ComponentKind::AndGate
                | ComponentKind::OrGate
                | ComponentKind::NandGate
                | ComponentKind::NorGate
                | ComponentKind::XorGate
                | ComponentKind::XnorGate
                | ComponentKind::Inverter
                | ComponentKind::BufferGate
                | ComponentKind::HalfAdder
                | ComponentKind::FullAdder
        )
    }

    /// Performs technology mapping and synthesis from schematic components and wires into physical chips.
    pub fn map_schematic(
        components: &[SchematicComponent],
        wires: &[SchematicWire],
        options: &TechMappingOptions,
    ) -> Result<MappedPhysicalBoard, SynthesisError> {
        if components.is_empty() {
            return Err(SynthesisError::EmptySchematic);
        }

        // 1. Validation check: If explosion is disabled, any abstract gate causes explicit failure
        if !options.auto_explode_gates {
            for comp in components {
                if Self::is_abstract_gate(comp.kind) {
                    return Err(SynthesisError::AbstractGateNotExploded {
                        designator: comp.name.clone(),
                        kind: comp.kind,
                    });
                }
            }
        }

        // 2. Build net name index from schematic wires and component pins
        let mut pin_to_net: HashMap<(usize, usize), String> = HashMap::new();

        // Map schematic pins to shared net names
        for (w_idx, wire) in wires.iter().enumerate() {
            let net_name = format!("NET_{}", w_idx + 1);
            for seg in &wire.segments {
                for comp in components {
                    for (pin_idx, pin_pos) in comp.all_pins().iter().map(|&(_, pos)| pos).enumerate() {
                        let d1 = (pin_pos - seg.start).length();
                        let d2 = (pin_pos - seg.end).length();
                        if d1 < 2.0 || d2 < 2.0 {
                            pin_to_net.insert((comp.id, pin_idx), net_name.clone());
                        }
                    }
                }
            }
        }

        let mut chips = Vec::new();
        let mut net_connections = Vec::new();
        let mut chip_id_gen = 1usize;
        let mut exploded_count = 0usize;

        // 3. Process each schematic component
        for comp in components {
            // Coordinate scale: map schematic grid pixels (e.g. 20px) to PCB millimeters (e.g. 2.54mm pitch)
            let board_x = (comp.pos.x / 20.0) * 2.54;
            let board_y = (comp.pos.y / 20.0) * 2.54;
            let base_pos = Pos2::new(board_x, board_y);

            match comp.kind {
                // Concrete passives -> SMD 0805
                ComponentKind::Resistor
                | ComponentKind::Capacitor
                | ComponentKind::Inductor
                | ComponentKind::Diode
                | ComponentKind::ZenerDiode
                | ComponentKind::SchottkyDiode => {
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        &comp.name,
                        format!("{:?}", comp.kind),
                        PackageType::Smd0805,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    // Net connections for pin 1 and pin 2
                    let net1 = pin_to_net
                        .get(&(comp.id, 0))
                        .cloned()
                        .unwrap_or_else(|| format!("NET_NC_{}_{}", comp.id, 1));
                    let net2 = pin_to_net
                        .get(&(comp.id, 1))
                        .cloned()
                        .unwrap_or_else(|| format!("NET_NC_{}_{}", comp.id, 2));

                    net_connections.push(PhysicalNetConnection {
                        net_name: net1,
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net2,
                        chip_id: cid,
                        pin_number: 2,
                        is_power_or_gnd: false,
                    });
                }

                // Discrete semiconductors -> TO-92
                ComponentKind::BjtNpn
                | ComponentKind::BjtPnp
                | ComponentKind::Nmos
                | ComponentKind::Pmos => {
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        &comp.name,
                        format!("{:?}", comp.kind),
                        PackageType::To92,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    for pin_num in 1..=3 {
                        let net = pin_to_net
                            .get(&(comp.id, pin_num - 1))
                            .cloned()
                            .unwrap_or_else(|| format!("NET_NC_{}_{}", comp.id, pin_num));
                        net_connections.push(PhysicalNetConnection {
                            net_name: net,
                            chip_id: cid,
                            pin_number: pin_num,
                            is_power_or_gnd: false,
                        });
                    }
                }

                // 555 Timer -> DIP-8
                ComponentKind::Timer555 => {
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        &comp.name,
                        "LM555N",
                        PackageType::Dip8,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    for pin_num in 1..=8 {
                        let is_pwr = pin_num == 1 || pin_num == 8;
                        let net = if pin_num == 1 {
                            "GND".to_string()
                        } else if pin_num == 8 {
                            "VCC".to_string()
                        } else {
                            pin_to_net
                                .get(&(comp.id, pin_num - 1))
                                .cloned()
                                .unwrap_or_else(|| format!("NET_555_{}_{}", comp.id, pin_num))
                        };
                        net_connections.push(PhysicalNetConnection {
                            net_name: net,
                            chip_id: cid,
                            pin_number: pin_num,
                            is_power_or_gnd: is_pwr,
                        });
                    }
                }

                // Power & Sources -> Pin Header 2
                ComponentKind::VoltageSource
                | ComponentKind::AcVoltageSource
                | ComponentKind::PulseGenerator
                | ComponentKind::ClockSource => {
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        &comp.name,
                        "HDR_PWR_2PIN",
                        PackageType::PinHeader2,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    net_connections.push(PhysicalNetConnection {
                        net_name: "VCC".to_string(),
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: true,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: "GND".to_string(),
                        chip_id: cid,
                        pin_number: 2,
                        is_power_or_gnd: true,
                    });
                }

                // Ground Reference -> Single Pin / Header
                ComponentKind::Ground => {
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        &comp.name,
                        "GND_PAD",
                        PackageType::PinHeader2,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    net_connections.push(PhysicalNetConnection {
                        net_name: "GND".to_string(),
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: true,
                    });
                }

                // Logic Probe -> Pin Header / Test Point
                ComponentKind::LogicProbe => {
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        &comp.name,
                        "TP_PROBE",
                        PackageType::PinHeader2,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    let net = pin_to_net
                        .get(&(comp.id, 0))
                        .cloned()
                        .unwrap_or_else(|| "GND".to_string());
                    net_connections.push(PhysicalNetConnection {
                        net_name: net,
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                }

                // 4. Abstract Logic Gates Exploded into Real Physical 7400-series Chips
                ComponentKind::XorGate => {
                    exploded_count += 1;
                    if options.force_universal_nand_explosion {
                        // Decompose XOR into 4 NAND gates inside a 74HC00 DIP-14 chip
                        // A XOR B = (A NAND (A NAND B)) NAND (B NAND (A NAND B))
                        // Or explode into multiple distinct 74HC00 physical packages to test de-clumping!
                        let chip = FootprintInstance::new(
                            chip_id_gen,
                            format!("{}_74HC00", comp.name),
                            "74HC00N (Quad 2-In NAND)",
                            PackageType::Dip14,
                            base_pos,
                        );
                        let cid = chip_id_gen;
                        chips.push(chip);
                        chip_id_gen += 1;

                        // Power pins
                        net_connections.push(PhysicalNetConnection {
                            net_name: "GND".to_string(),
                            chip_id: cid,
                            pin_number: 7,
                            is_power_or_gnd: true,
                        });
                        net_connections.push(PhysicalNetConnection {
                            net_name: "VCC".to_string(),
                            chip_id: cid,
                            pin_number: 14,
                            is_power_or_gnd: true,
                        });

                        // Logic inputs and output
                        let net_a = pin_to_net
                            .get(&(comp.id, 0))
                            .cloned()
                            .unwrap_or_else(|| format!("NET_XOR_{}_A", comp.id));
                        let net_b = pin_to_net
                            .get(&(comp.id, 1))
                            .cloned()
                            .unwrap_or_else(|| format!("NET_XOR_{}_B", comp.id));
                        let net_out = pin_to_net
                            .get(&(comp.id, 2))
                            .cloned()
                            .unwrap_or_else(|| format!("NET_XOR_{}_OUT", comp.id));

                        net_connections.push(PhysicalNetConnection {
                            net_name: net_a,
                            chip_id: cid,
                            pin_number: 1,
                            is_power_or_gnd: false,
                        });
                        net_connections.push(PhysicalNetConnection {
                            net_name: net_b,
                            chip_id: cid,
                            pin_number: 2,
                            is_power_or_gnd: false,
                        });
                        net_connections.push(PhysicalNetConnection {
                            net_name: net_out,
                            chip_id: cid,
                            pin_number: 3,
                            is_power_or_gnd: false,
                        });
                    } else {
                        // Map directly to 74HC86 Quad 2-Input XOR DIP-14
                        let chip = FootprintInstance::new(
                            chip_id_gen,
                            format!("{}_74HC86", comp.name),
                            "74HC86N (Quad 2-In XOR)",
                            PackageType::Dip14,
                            base_pos,
                        );
                        let cid = chip_id_gen;
                        chips.push(chip);
                        chip_id_gen += 1;

                        net_connections.push(PhysicalNetConnection {
                            net_name: "GND".to_string(),
                            chip_id: cid,
                            pin_number: 7,
                            is_power_or_gnd: true,
                        });
                        net_connections.push(PhysicalNetConnection {
                            net_name: "VCC".to_string(),
                            chip_id: cid,
                            pin_number: 14,
                            is_power_or_gnd: true,
                        });

                        let net_a = pin_to_net
                            .get(&(comp.id, 0))
                            .cloned()
                            .unwrap_or_else(|| format!("NET_A_{}", comp.id));
                        let net_b = pin_to_net
                            .get(&(comp.id, 1))
                            .cloned()
                            .unwrap_or_else(|| format!("NET_B_{}", comp.id));
                        let net_out = pin_to_net
                            .get(&(comp.id, 2))
                            .cloned()
                            .unwrap_or_else(|| format!("NET_OUT_{}", comp.id));

                        net_connections.push(PhysicalNetConnection {
                            net_name: net_a,
                            chip_id: cid,
                            pin_number: 1,
                            is_power_or_gnd: false,
                        });
                        net_connections.push(PhysicalNetConnection {
                            net_name: net_b,
                            chip_id: cid,
                            pin_number: 2,
                            is_power_or_gnd: false,
                        });
                        net_connections.push(PhysicalNetConnection {
                            net_name: net_out,
                            chip_id: cid,
                            pin_number: 3,
                            is_power_or_gnd: false,
                        });
                    }
                }

                ComponentKind::AndGate => {
                    exploded_count += 1;
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        format!("{}_74HC08", comp.name),
                        "74HC08N (Quad 2-In AND)",
                        PackageType::Dip14,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    net_connections.push(PhysicalNetConnection {
                        net_name: "GND".to_string(),
                        chip_id: cid,
                        pin_number: 7,
                        is_power_or_gnd: true,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: "VCC".to_string(),
                        chip_id: cid,
                        pin_number: 14,
                        is_power_or_gnd: true,
                    });

                    let net_a = pin_to_net.get(&(comp.id, 0)).cloned().unwrap_or_default();
                    let net_b = pin_to_net.get(&(comp.id, 1)).cloned().unwrap_or_default();
                    let net_out = pin_to_net.get(&(comp.id, 2)).cloned().unwrap_or_default();

                    net_connections.push(PhysicalNetConnection {
                        net_name: net_a,
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_b,
                        chip_id: cid,
                        pin_number: 2,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_out,
                        chip_id: cid,
                        pin_number: 3,
                        is_power_or_gnd: false,
                    });
                }

                ComponentKind::OrGate => {
                    exploded_count += 1;
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        format!("{}_74HC32", comp.name),
                        "74HC32N (Quad 2-In OR)",
                        PackageType::Dip14,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    net_connections.push(PhysicalNetConnection {
                        net_name: "GND".to_string(),
                        chip_id: cid,
                        pin_number: 7,
                        is_power_or_gnd: true,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: "VCC".to_string(),
                        chip_id: cid,
                        pin_number: 14,
                        is_power_or_gnd: true,
                    });

                    let net_a = pin_to_net.get(&(comp.id, 0)).cloned().unwrap_or_default();
                    let net_b = pin_to_net.get(&(comp.id, 1)).cloned().unwrap_or_default();
                    let net_out = pin_to_net.get(&(comp.id, 2)).cloned().unwrap_or_default();

                    net_connections.push(PhysicalNetConnection {
                        net_name: net_a,
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_b,
                        chip_id: cid,
                        pin_number: 2,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_out,
                        chip_id: cid,
                        pin_number: 3,
                        is_power_or_gnd: false,
                    });
                }

                ComponentKind::Inverter => {
                    exploded_count += 1;
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        format!("{}_74HC04", comp.name),
                        "74HC04N (Hex Inverter)",
                        PackageType::Dip14,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    net_connections.push(PhysicalNetConnection {
                        net_name: "GND".to_string(),
                        chip_id: cid,
                        pin_number: 7,
                        is_power_or_gnd: true,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: "VCC".to_string(),
                        chip_id: cid,
                        pin_number: 14,
                        is_power_or_gnd: true,
                    });

                    let net_in = pin_to_net.get(&(comp.id, 0)).cloned().unwrap_or_default();
                    let net_out = pin_to_net.get(&(comp.id, 1)).cloned().unwrap_or_default();

                    net_connections.push(PhysicalNetConnection {
                        net_name: net_in,
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_out,
                        chip_id: cid,
                        pin_number: 2,
                        is_power_or_gnd: false,
                    });
                }

                ComponentKind::NandGate => {
                    exploded_count += 1;
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        format!("{}_74HC00", comp.name),
                        "74HC00N (Quad 2-In NAND)",
                        PackageType::Dip14,
                        base_pos,
                    );
                    let cid = chip_id_gen;
                    chips.push(chip);
                    chip_id_gen += 1;

                    net_connections.push(PhysicalNetConnection {
                        net_name: "GND".to_string(),
                        chip_id: cid,
                        pin_number: 7,
                        is_power_or_gnd: true,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: "VCC".to_string(),
                        chip_id: cid,
                        pin_number: 14,
                        is_power_or_gnd: true,
                    });

                    let net_a = pin_to_net.get(&(comp.id, 0)).cloned().unwrap_or_default();
                    let net_b = pin_to_net.get(&(comp.id, 1)).cloned().unwrap_or_default();
                    let net_out = pin_to_net.get(&(comp.id, 2)).cloned().unwrap_or_default();

                    net_connections.push(PhysicalNetConnection {
                        net_name: net_a,
                        chip_id: cid,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_b,
                        chip_id: cid,
                        pin_number: 2,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_out,
                        chip_id: cid,
                        pin_number: 3,
                        is_power_or_gnd: false,
                    });
                }

                ComponentKind::HalfAdder => {
                    exploded_count += 2;
                    // Half adder explodes into 2 chips: 74HC86 (XOR for SUM) and 74HC08 (AND for CARRY)
                    // Initially placed at the EXACT same position, creating a clump to be relaxed!
                    let xor_chip = FootprintInstance::new(
                        chip_id_gen,
                        format!("{}_SUM_XOR", comp.name),
                        "74HC86N (SUM Gate)",
                        PackageType::Dip14,
                        base_pos,
                    );
                    let xor_id = chip_id_gen;
                    chips.push(xor_chip);
                    chip_id_gen += 1;

                    let and_chip = FootprintInstance::new(
                        chip_id_gen,
                        format!("{}_COUT_AND", comp.name),
                        "74HC08N (CARRY Gate)",
                        PackageType::Dip14,
                        base_pos, // Clumped on top of xor_chip
                    );
                    let and_id = chip_id_gen;
                    chips.push(and_chip);
                    chip_id_gen += 1;

                    // Power pins
                    for cid in [xor_id, and_id] {
                        net_connections.push(PhysicalNetConnection {
                            net_name: "GND".to_string(),
                            chip_id: cid,
                            pin_number: 7,
                            is_power_or_gnd: true,
                        });
                        net_connections.push(PhysicalNetConnection {
                            net_name: "VCC".to_string(),
                            chip_id: cid,
                            pin_number: 14,
                            is_power_or_gnd: true,
                        });
                    }

                    let net_a = pin_to_net.get(&(comp.id, 0)).cloned().unwrap_or_default();
                    let net_b = pin_to_net.get(&(comp.id, 1)).cloned().unwrap_or_default();
                    let net_sum = pin_to_net.get(&(comp.id, 2)).cloned().unwrap_or_default();
                    let net_cout = pin_to_net.get(&(comp.id, 3)).cloned().unwrap_or_default();

                    // Connect A & B to both chips
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_a.clone(),
                        chip_id: xor_id,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_b.clone(),
                        chip_id: xor_id,
                        pin_number: 2,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_sum,
                        chip_id: xor_id,
                        pin_number: 3,
                        is_power_or_gnd: false,
                    });

                    net_connections.push(PhysicalNetConnection {
                        net_name: net_a,
                        chip_id: and_id,
                        pin_number: 1,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_b,
                        chip_id: and_id,
                        pin_number: 2,
                        is_power_or_gnd: false,
                    });
                    net_connections.push(PhysicalNetConnection {
                        net_name: net_cout,
                        chip_id: and_id,
                        pin_number: 3,
                        is_power_or_gnd: false,
                    });
                }

                // Other generic components -> map to DIP-14 or Header
                _ => {
                    let chip = FootprintInstance::new(
                        chip_id_gen,
                        &comp.name,
                        format!("{:?}", comp.kind),
                        PackageType::Dip14,
                        base_pos,
                    );
                    chips.push(chip);
                    chip_id_gen += 1;
                }
            }
        }

        Ok(MappedPhysicalBoard {
            chips,
            net_connections,
            exploded_count,
        })
    }
}
