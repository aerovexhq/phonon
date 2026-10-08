#![deny(unsafe_code)]

//! Reusable modular subcircuit packaging (`.phnc`) and hierarchical SPICE `.SUBCKT` compilation engine.

use super::binary_format::{read_component, read_wire, write_component, write_wire};
use super::components::{ComponentKind, SchematicComponent};
use super::wire::SchematicWire;
use std::collections::{HashMap, HashSet};

/// Port signal flow direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
    Power,
    Ground,
}

impl PortDirection {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Bidirectional => "Bidirectional",
            Self::Power => "Power",
            Self::Ground => "Ground",
        }
    }
}

/// Physical symbol perimeter edge where the port pin is positioned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl PortEdge {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
        }
    }
}

/// Boundary connection terminal port exposed by a packaged subcircuit.
#[derive(Debug, Clone, PartialEq)]
pub struct SubcircuitPort {
    pub name: String,
    pub direction: PortDirection,
    pub edge: PortEdge,
    pub internal_net: String,
}

impl SubcircuitPort {
    pub fn new(name: &str, direction: PortDirection, edge: PortEdge, internal_net: &str) -> Self {
        Self {
            name: name.to_string(),
            direction,
            edge,
            internal_net: internal_net.to_string(),
        }
    }
}

/// Self-contained reusable schematic subcircuit package (`.phnc`).
#[derive(Debug, Clone, PartialEq)]
pub struct SubcircuitPackage {
    pub name: String,
    pub description: String,
    pub version: u32,
    pub author: String,
    pub ports: Vec<SubcircuitPort>,
    pub components: Vec<SchematicComponent>,
    pub wires: Vec<SchematicWire>,
}

impl Default for SubcircuitPackage {
    fn default() -> Self {
        Self {
            name: "NewSubcircuit".to_string(),
            description: "Modular subcircuit block".to_string(),
            version: 1,
            author: "Phonon User".to_string(),
            ports: Vec::new(),
            components: Vec::new(),
            wires: Vec::new(),
        }
    }
}

impl SubcircuitPackage {
    pub fn new(
        name: &str,
        description: &str,
        ports: Vec<SubcircuitPort>,
        components: Vec<SchematicComponent>,
        wires: Vec<SchematicWire>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            version: 1,
            author: "Phonon User".to_string(),
            ports,
            components,
            wires,
        }
    }

    /// Compiles this subcircuit into a standard SPICE `.SUBCKT ... .ENDS` definition.
    pub fn to_spice_subckt(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("* Subcircuit Package: {}", self.name));
        lines.push(format!("* Description: {}", self.description));

        // Format port list
        let port_names: Vec<String> = self.ports.iter().map(|p| p.name.clone()).collect();
        lines.push(format!(".SUBCKT {} {}", self.name, port_names.join(" ")));

        // Map internal nets to subcircuit ports where applicable
        let mut net_mapping: HashMap<String, String> = HashMap::new();
        for port in &self.ports {
            net_mapping.insert(port.internal_net.clone(), port.name.clone());
        }

        // Map internal components to SPICE lines
        for comp in &self.components {
            match comp.kind {
                ComponentKind::Ground => {}
                ComponentKind::Resistor => {
                    let n1 = net_mapping.get("1").cloned().unwrap_or_else(|| "N1".to_string());
                    let n2 = net_mapping.get("2").cloned().unwrap_or_else(|| "N2".to_string());
                    lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                }
                ComponentKind::Capacitor => {
                    let n1 = net_mapping.get("1").cloned().unwrap_or_else(|| "N1".to_string());
                    let n2 = net_mapping.get("2").cloned().unwrap_or_else(|| "N2".to_string());
                    lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                }
                ComponentKind::Inductor => {
                    let n1 = net_mapping.get("1").cloned().unwrap_or_else(|| "N1".to_string());
                    let n2 = net_mapping.get("2").cloned().unwrap_or_else(|| "N2".to_string());
                    lines.push(format!("{} {} {} {}", comp.name, n1, n2, comp.value_str));
                }
                ComponentKind::Diode => {
                    let n1 = net_mapping.get("A").cloned().unwrap_or_else(|| "NA".to_string());
                    let n2 = net_mapping.get("K").cloned().unwrap_or_else(|| "NK".to_string());
                    lines.push(format!("{} {} {} 1N4148", comp.name, n1, n2));
                }
                _ => {
                    lines.push(format!("* Component {} ({:?})", comp.name, comp.kind));
                }
            }
        }

        lines.push(format!(".ENDS {}", self.name));
        lines.join("\n")
    }

    /// Computes Adler-32 checksum.
    fn compute_adler32(data: &[u8]) -> u32 {
        let mut a: u32 = 1;
        let mut b: u32 = 0;
        for &byte in data {
            a = (a + byte as u32) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }

    /// Serializes subcircuit package into binary format (`.phnc`).
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        // 1. Metadata strings
        let name_bytes = self.name.as_bytes();
        payload.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        payload.extend_from_slice(name_bytes);

        let desc_bytes = self.description.as_bytes();
        payload.extend_from_slice(&(desc_bytes.len() as u16).to_le_bytes());
        payload.extend_from_slice(desc_bytes);

        let author_bytes = self.author.as_bytes();
        payload.extend_from_slice(&(author_bytes.len() as u16).to_le_bytes());
        payload.extend_from_slice(author_bytes);

        payload.extend_from_slice(&self.version.to_le_bytes());

        // 2. Ports
        payload.extend_from_slice(&(self.ports.len() as u16).to_le_bytes());
        for p in &self.ports {
            let pname_bytes = p.name.as_bytes();
            payload.extend_from_slice(&(pname_bytes.len() as u16).to_le_bytes());
            payload.extend_from_slice(pname_bytes);

            let dir_byte = match p.direction {
                PortDirection::Input => 0x01,
                PortDirection::Output => 0x02,
                PortDirection::Bidirectional => 0x03,
                PortDirection::Power => 0x04,
                PortDirection::Ground => 0x05,
            };
            payload.push(dir_byte);

            let edge_byte = match p.edge {
                PortEdge::Left => 0x01,
                PortEdge::Right => 0x02,
                PortEdge::Top => 0x03,
                PortEdge::Bottom => 0x04,
            };
            payload.push(edge_byte);

            let inet_bytes = p.internal_net.as_bytes();
            payload.extend_from_slice(&(inet_bytes.len() as u16).to_le_bytes());
            payload.extend_from_slice(inet_bytes);
        }

        // 3. Components
        payload.extend_from_slice(&(self.components.len() as u32).to_le_bytes());
        for comp in &self.components {
            write_component(comp, &mut payload);
        }

        // 4. Wires
        payload.extend_from_slice(&(self.wires.len() as u32).to_le_bytes());
        for wire in &self.wires {
            write_wire(wire, &mut payload);
        }

        // Header: "PHNC" (4 bytes) + version 1 (u16) + checksum (u32) + payload
        let checksum = Self::compute_adler32(&payload);
        let mut final_buf = Vec::with_capacity(10 + payload.len());
        final_buf.extend_from_slice(b"PHNC");
        final_buf.extend_from_slice(&1u16.to_le_bytes());
        final_buf.extend_from_slice(&checksum.to_le_bytes());
        final_buf.extend_from_slice(&payload);
        final_buf
    }

    /// Deserializes subcircuit package from binary bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        if data.len() < 10 {
            return Err("Binary package payload too short".to_string());
        }
        if &data[0..4] != b"PHNC" {
            return Err("Invalid magic bytes: expected PHNC".to_string());
        }
        let format_ver = u16::from_le_bytes([data[4], data[5]]);
        if format_ver != 1 {
            return Err(format!("Unsupported format version: {}", format_ver));
        }
        let expected_checksum = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        let payload = &data[10..];
        let actual_checksum = Self::compute_adler32(payload);
        if actual_checksum != expected_checksum {
            return Err(format!(
                "Checksum mismatch: expected 0x{:08X}, got 0x{:08X}",
                expected_checksum, actual_checksum
            ));
        }

        let mut offset = 0;
        let read_str = |buf: &[u8], off: &mut usize| -> Result<String, String> {
            if *off + 2 > buf.len() {
                return Err("Truncated string length".to_string());
            }
            let len = u16::from_le_bytes([buf[*off], buf[*off + 1]]) as usize;
            *off += 2;
            if *off + len > buf.len() {
                return Err("Truncated string data".to_string());
            }
            let s = std::str::from_utf8(&buf[*off..*off + len])
                .map_err(|e| e.to_string())?
                .to_string();
            *off += len;
            Ok(s)
        };

        let name = read_str(payload, &mut offset)?;
        let description = read_str(payload, &mut offset)?;
        let author = read_str(payload, &mut offset)?;

        if offset + 4 > payload.len() {
            return Err("Truncated version".to_string());
        }
        let version = u32::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
        ]);
        offset += 4;

        if offset + 2 > payload.len() {
            return Err("Truncated port count".to_string());
        }
        let port_count = u16::from_le_bytes([payload[offset], payload[offset + 1]]) as usize;
        offset += 2;

        let mut ports = Vec::with_capacity(port_count);
        for _ in 0..port_count {
            let pname = read_str(payload, &mut offset)?;
            if offset + 2 > payload.len() {
                return Err("Truncated port direction/edge".to_string());
            }
            let dir_byte = payload[offset];
            offset += 1;
            let edge_byte = payload[offset];
            offset += 1;

            let direction = match dir_byte {
                0x01 => PortDirection::Input,
                0x02 => PortDirection::Output,
                0x03 => PortDirection::Bidirectional,
                0x04 => PortDirection::Power,
                0x05 => PortDirection::Ground,
                _ => PortDirection::Bidirectional,
            };

            let edge = match edge_byte {
                0x01 => PortEdge::Left,
                0x02 => PortEdge::Right,
                0x03 => PortEdge::Top,
                0x04 => PortEdge::Bottom,
                _ => PortEdge::Left,
            };

            let inet = read_str(payload, &mut offset)?;
            ports.push(SubcircuitPort {
                name: pname,
                direction,
                edge,
                internal_net: inet,
            });
        }

        if offset + 4 > payload.len() {
            return Err("Truncated component count".to_string());
        }
        let comp_count = u32::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
        ]) as usize;
        offset += 4;

        let mut components = Vec::with_capacity(comp_count);
        for _ in 0..comp_count {
            let comp = read_component(payload, &mut offset).map_err(|e| e.to_string())?;
            components.push(comp);
        }

        if offset + 4 > payload.len() {
            return Err("Truncated wire count".to_string());
        }
        let wire_count = u32::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
        ]) as usize;
        offset += 4;

        let mut wires = Vec::with_capacity(wire_count);
        for _ in 0..wire_count {
            let wire = read_wire(payload, &mut offset).map_err(|e| e.to_string())?;
            wires.push(wire);
        }

        Ok(SubcircuitPackage {
            name,
            description,
            version,
            author,
            ports,
            components,
            wires,
        })
    }
}

/// Discovers boundary connection ports for a selected collection of components and wires.
pub fn discover_boundary_ports(
    components: &[SchematicComponent],
    wires: &[SchematicWire],
) -> Vec<SubcircuitPort> {
    let mut ports = Vec::new();
    let mut connected_endpoints = HashSet::new();

    // Collect all wire endpoints
    for wire in wires {
        let p1 = (
            (wire.start_point().x * 2.0).round() as i32,
            (wire.start_point().y * 2.0).round() as i32,
        );
        let p2 = (
            (wire.end_point().x * 2.0).round() as i32,
            (wire.end_point().y * 2.0).round() as i32,
        );
        connected_endpoints.insert(p1);
        connected_endpoints.insert(p2);
    }

    // Also collect component pin locations
    let mut pin_locations = HashSet::new();
    for comp in components {
        for (_, pin_pos) in comp.all_pins() {
            pin_locations.insert((
                (pin_pos.x * 2.0).round() as i32,
                (pin_pos.y * 2.0).round() as i32,
            ));
        }
    }

    // Check for wires with explicit net names that have open boundary ends
    let mut seen_wire_nets = HashSet::new();
    for wire in wires {
        if let Some(net) = &wire.net_name {
            if !net.is_empty() && seen_wire_nets.insert(net.clone()) {
                let start_q = (
                    (wire.start_point().x * 2.0).round() as i32,
                    (wire.start_point().y * 2.0).round() as i32,
                );
                let end_q = (
                    (wire.end_point().x * 2.0).round() as i32,
                    (wire.end_point().y * 2.0).round() as i32,
                );
                let is_start_open = !pin_locations.contains(&start_q);
                let is_end_open = !pin_locations.contains(&end_q);

                if is_start_open || is_end_open {
                    let edge = if is_start_open && wire.start_point().x <= wire.end_point().x {
                        PortEdge::Left
                    } else if is_end_open && wire.end_point().x >= wire.start_point().x {
                        PortEdge::Right
                    } else {
                        PortEdge::Top
                    };

                    let direction = if net.to_uppercase().contains("IN") {
                        PortDirection::Input
                    } else if net.to_uppercase().contains("OUT") {
                        PortDirection::Output
                    } else if net.to_uppercase() == "GND" || net == "0" {
                        PortDirection::Ground
                    } else {
                        PortDirection::Bidirectional
                    };

                    ports.push(SubcircuitPort::new(
                        net,
                        direction,
                        edge,
                        net,
                    ));
                }
            }
        }
    }

    let mut in_count = 1;
    let mut out_count = 1;

    for comp in components {
        for (pin_name, pin_pos) in comp.all_pins() {
            let qpos = (
                (pin_pos.x * 2.0).round() as i32,
                (pin_pos.y * 2.0).round() as i32,
            );

            // If pin is either not touched by internal wires or has external port characteristics
            let is_ground = comp.kind == ComponentKind::Ground;
            let is_vsource = comp.kind == ComponentKind::VoltageSource;

            if is_ground {
                ports.push(SubcircuitPort::new(
                    "GND",
                    PortDirection::Ground,
                    PortEdge::Bottom,
                    "0",
                ));
            } else if is_vsource {
                ports.push(SubcircuitPort::new(
                    "VCC",
                    PortDirection::Power,
                    PortEdge::Top,
                    &format!("{}_{}", comp.name, pin_name),
                ));
            } else if !connected_endpoints.contains(&qpos) {
                // Unconnected external pin
                let (pname, dir, edge) = if pin_name == "1" || pin_name == "IN" || pin_name == "A" {
                    let name = format!("IN_{}", in_count);
                    in_count += 1;
                    (name, PortDirection::Input, PortEdge::Left)
                } else {
                    let name = format!("OUT_{}", out_count);
                    out_count += 1;
                    (name, PortDirection::Output, PortEdge::Right)
                };

                ports.push(SubcircuitPort::new(
                    &pname,
                    dir,
                    edge,
                    &format!("{}_{}", comp.name, pin_name),
                ));
            }
        }
    }

    if ports.is_empty() {
        // Fallback default ports if all pins are connected internally
        ports.push(SubcircuitPort::new("IN", PortDirection::Input, PortEdge::Left, "1"));
        ports.push(SubcircuitPort::new("OUT", PortDirection::Output, PortEdge::Right, "2"));
    }

    ports
}

/// Registry managing active, imported, and built-in reusable subcircuit packages.
#[derive(Debug, Clone, Default)]
pub struct SubcircuitRegistry {
    pub packages: HashMap<String, SubcircuitPackage>,
}

impl SubcircuitRegistry {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Optional helper to register built-in demo packages for test harnesses or examples.
    pub fn register_built_in_defaults(&mut self) {
        // Built-in demo package: Voltage Divider
        let r1 = SchematicComponent::new(1, ComponentKind::Resistor, egui::pos2(100.0, 100.0), 0);
        let r2 = SchematicComponent::new(2, ComponentKind::Resistor, egui::pos2(100.0, 180.0), 0);
        let ports = vec![
            SubcircuitPort::new("IN", PortDirection::Input, PortEdge::Top, "net_in"),
            SubcircuitPort::new("OUT", PortDirection::Output, PortEdge::Right, "net_mid"),
            SubcircuitPort::new("GND", PortDirection::Ground, PortEdge::Bottom, "0"),
        ];
        let divider = SubcircuitPackage::new(
            "VoltageDivider",
            "Precision 2:1 resistive attenuator divider network",
            ports,
            vec![r1, r2],
            Vec::new(),
        );
        self.register(divider);
    }

    pub fn register(&mut self, package: SubcircuitPackage) {
        self.packages.insert(package.name.clone(), package);
    }

    pub fn get(&self, name: &str) -> Option<&SubcircuitPackage> {
        self.packages.get(name)
    }

    pub fn list(&self) -> Vec<&SubcircuitPackage> {
        let mut list: Vec<&SubcircuitPackage> = self.packages.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn remove(&mut self, name: &str) -> Option<SubcircuitPackage> {
        self.packages.remove(name)
    }
}
