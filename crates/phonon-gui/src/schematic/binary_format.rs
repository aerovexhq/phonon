#![deny(unsafe_code)]

//! Ultra-compact binary project serialization format (.phn) for Phonon Studio.
//!
//! Serialization protocol:
//! - Header (24 bytes):
//!   * Magic: `PHONON\x01` (8 bytes)
//!   * Version: `u16` little endian (1)
//!   * Flags: `u16` little endian (0)
//!   * Component count: `u32` little endian
//!   * Wire count: `u32` little endian
//!   * Title byte length: `u32` little endian
//! - Title bytes (UTF-8)
//! - Components (packed records):
//!   * `id`: `u32` little endian
//!   * `kind_discriminant`: `u16` little endian
//!   * `category_discriminant`: `u8`
//!   * `pos_x`: `f32` little endian
//!   * `pos_y`: `f32` little endian
//!   * `rotation`: `u8` (0, 1, 2, 3)
//!   * `value_len`: `u16` little endian
//!   * `value_bytes`: UTF-8
//!   * `properties_count`: `u16` little endian
//!   * For each property: `key_len` (u16) + `key_bytes` + `val_len` (u16) + `val_bytes`
//! - Wires (packed records):
//!   * `id`: `u32` little endian
//!   * `start_x`: `f32`, `start_y`: `f32` little endian
//!   * `end_x`: `f32`, `end_y`: `f32` little endian
//!   * `net_name_len`: `u16` little endian
//!   * `net_name_bytes`: UTF-8
//! - Footer (4 bytes):
//!   * Adler-32 checksum of all preceding payload bytes (`u32` little endian).

use std::path::Path;

use egui::Pos2;

use super::categories::ComponentCategory;
use super::components::{ComponentKind, SchematicComponent};
use super::wire::SchematicWire;

/// 8-byte magic header constant designating a valid Phonon binary project stream.
pub const PHONON_MAGIC: [u8; 8] = *b"PHONON\x01\0";

/// Current binary specification version.
pub const CURRENT_VERSION: u16 = 1;

/// Deserialized project payload containing title, components, and wires.
#[derive(Debug, Clone, PartialEq)]
pub struct DeserializedProject {
    pub title: String,
    pub components: Vec<SchematicComponent>,
    pub wires: Vec<SchematicWire>,
}

/// Errors occurring during binary format serialization, deserialization, or I/O.
#[derive(Debug, thiserror::Error)]
pub enum BinaryFormatError {
    #[error("invalid magic header bytes")]
    InvalidMagic,
    #[error("unsupported format version: {0}")]
    UnsupportedVersion(u16),
    #[error("truncated binary data: {0}")]
    TruncatedData(String),
    #[error("corrupt checksum footer")]
    CorruptChecksum,
    #[error("invalid utf-8 string: {0}")]
    Utf8Error(String),
    #[error("unknown component kind discriminant: {0}")]
    UnknownComponentKind(u16),
    #[error("i/o error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for BinaryFormatError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

/// Computes the 32-bit Adler-32 checksum of a byte slice.
pub fn compute_adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for chunk in data.chunks(5552) {
        for &byte in chunk {
            s1 += byte as u32;
            s2 += s1;
        }
        s1 %= MOD_ADLER;
        s2 %= MOD_ADLER;
    }
    (s2 << 16) | s1
}

/// Maps a `ComponentKind` to a stable 16-bit integer discriminant.
pub fn component_kind_to_discriminant(kind: ComponentKind) -> u16 {
    match kind {
        ComponentKind::Resistor => 0,
        ComponentKind::Capacitor => 1,
        ComponentKind::Inductor => 2,
        ComponentKind::Ground => 3,
        ComponentKind::Transformer => 4,
        ComponentKind::VoltageSource => 5,
        ComponentKind::AcVoltageSource => 6,
        ComponentKind::CurrentSource => 7,
        ComponentKind::PulseGenerator => 8,
        ComponentKind::Diode => 9,
        ComponentKind::ZenerDiode => 10,
        ComponentKind::Led => 11,
        ComponentKind::SchottkyDiode => 12,
        ComponentKind::Nmos => 13,
        ComponentKind::Pmos => 14,
        ComponentKind::FinFet => 15,
        ComponentKind::GaaNanosheet => 16,
        ComponentKind::BjtNpn => 17,
        ComponentKind::BjtPnp => 18,
        ComponentKind::OpAmp => 19,
        ComponentKind::Inverter => 20,
        ComponentKind::NandGate => 21,
        ComponentKind::NorGate => 22,
        ComponentKind::Mux2to1 => 23,
        ComponentKind::StrainGauge => 24,
        ComponentKind::TactileMatrix => 25,
        ComponentKind::Imu9Dof => 26,
        ComponentKind::SawIdt => 27,
        ComponentKind::MajoranaJunction => 28,
        ComponentKind::ParafermionicCavity => 29,
        ComponentKind::SkyrmionRouter => 30,
        ComponentKind::PhLungs => 31,
        ComponentKind::PhVocalFolds => 32,
        ComponentKind::PhVocalTract => 33,
        ComponentKind::PhLipRadiation => 34,
        ComponentKind::Potentiometer => 35,
        ComponentKind::SwitchSpst => 36,
        ComponentKind::PushButton => 37,
        ComponentKind::ClockSource => 38,
        ComponentKind::VddRail => 39,
        ComponentKind::BufferGate => 40,
        ComponentKind::AndGate => 41,
        ComponentKind::OrGate => 42,
        ComponentKind::XorGate => 43,
        ComponentKind::XnorGate => 44,
        ComponentKind::HalfAdder => 45,
        ComponentKind::FullAdder => 46,
        ComponentKind::Mux4to1 => 47,
        ComponentKind::Demux1to2 => 48,
        ComponentKind::DFlipFlop => 49,
        ComponentKind::SrLatch => 50,
        ComponentKind::Counter4Bit => 51,
        ComponentKind::Comparator => 52,
        ComponentKind::Timer555 => 53,
        ComponentKind::VoltageRegulator => 54,
        ComponentKind::LogicProbe => 55,
        ComponentKind::SevenSegment => 56,
        ComponentKind::Buzzer => 57,
        ComponentKind::Adder => 58,
        ComponentKind::Subtractor => 59,
        ComponentKind::Multiplier => 60,
        ComponentKind::Divider => 61,
        ComponentKind::ArithmeticLogicUnit => 62,
        ComponentKind::BitSplitter => 63,
        ComponentKind::BitMerger => 64,
        ComponentKind::BusTap => 65,
        ComponentKind::FloatAdder => 66,
        ComponentKind::FloatSubtractor => 67,
        ComponentKind::FloatMultiplier => 68,
        ComponentKind::FloatDivider => 69,
        ComponentKind::FloatComparator => 70,
    }
}

/// Maps a 16-bit integer discriminant to a `ComponentKind`.
pub fn component_kind_from_discriminant(d: u16) -> Result<ComponentKind, BinaryFormatError> {
    match d {
        0 => Ok(ComponentKind::Resistor),
        1 => Ok(ComponentKind::Capacitor),
        2 => Ok(ComponentKind::Inductor),
        3 => Ok(ComponentKind::Ground),
        4 => Ok(ComponentKind::Transformer),
        5 => Ok(ComponentKind::VoltageSource),
        6 => Ok(ComponentKind::AcVoltageSource),
        7 => Ok(ComponentKind::CurrentSource),
        8 => Ok(ComponentKind::PulseGenerator),
        9 => Ok(ComponentKind::Diode),
        10 => Ok(ComponentKind::ZenerDiode),
        11 => Ok(ComponentKind::Led),
        12 => Ok(ComponentKind::SchottkyDiode),
        13 => Ok(ComponentKind::Nmos),
        14 => Ok(ComponentKind::Pmos),
        15 => Ok(ComponentKind::FinFet),
        16 => Ok(ComponentKind::GaaNanosheet),
        17 => Ok(ComponentKind::BjtNpn),
        18 => Ok(ComponentKind::BjtPnp),
        19 => Ok(ComponentKind::OpAmp),
        20 => Ok(ComponentKind::Inverter),
        21 => Ok(ComponentKind::NandGate),
        22 => Ok(ComponentKind::NorGate),
        23 => Ok(ComponentKind::Mux2to1),
        24 => Ok(ComponentKind::StrainGauge),
        25 => Ok(ComponentKind::TactileMatrix),
        26 => Ok(ComponentKind::Imu9Dof),
        27 => Ok(ComponentKind::SawIdt),
        28 => Ok(ComponentKind::MajoranaJunction),
        29 => Ok(ComponentKind::ParafermionicCavity),
        30 => Ok(ComponentKind::SkyrmionRouter),
        31 => Ok(ComponentKind::PhLungs),
        32 => Ok(ComponentKind::PhVocalFolds),
        33 => Ok(ComponentKind::PhVocalTract),
        34 => Ok(ComponentKind::PhLipRadiation),
        35 => Ok(ComponentKind::Potentiometer),
        36 => Ok(ComponentKind::SwitchSpst),
        37 => Ok(ComponentKind::PushButton),
        38 => Ok(ComponentKind::ClockSource),
        39 => Ok(ComponentKind::VddRail),
        40 => Ok(ComponentKind::BufferGate),
        41 => Ok(ComponentKind::AndGate),
        42 => Ok(ComponentKind::OrGate),
        43 => Ok(ComponentKind::XorGate),
        44 => Ok(ComponentKind::XnorGate),
        45 => Ok(ComponentKind::HalfAdder),
        46 => Ok(ComponentKind::FullAdder),
        47 => Ok(ComponentKind::Mux4to1),
        48 => Ok(ComponentKind::Demux1to2),
        49 => Ok(ComponentKind::DFlipFlop),
        50 => Ok(ComponentKind::SrLatch),
        51 => Ok(ComponentKind::Counter4Bit),
        52 => Ok(ComponentKind::Comparator),
        53 => Ok(ComponentKind::Timer555),
        54 => Ok(ComponentKind::VoltageRegulator),
        55 => Ok(ComponentKind::LogicProbe),
        56 => Ok(ComponentKind::SevenSegment),
        57 => Ok(ComponentKind::Buzzer),
        58 => Ok(ComponentKind::Adder),
        59 => Ok(ComponentKind::Subtractor),
        60 => Ok(ComponentKind::Multiplier),
        61 => Ok(ComponentKind::Divider),
        62 => Ok(ComponentKind::ArithmeticLogicUnit),
        63 => Ok(ComponentKind::BitSplitter),
        64 => Ok(ComponentKind::BitMerger),
        65 => Ok(ComponentKind::BusTap),
        66 => Ok(ComponentKind::FloatAdder),
        67 => Ok(ComponentKind::FloatSubtractor),
        68 => Ok(ComponentKind::FloatMultiplier),
        69 => Ok(ComponentKind::FloatDivider),
        70 => Ok(ComponentKind::FloatComparator),
        other => Err(BinaryFormatError::UnknownComponentKind(other)),
    }
}

/// Maps a `ComponentCategory` to an 8-bit integer discriminant.
pub fn component_category_to_discriminant(cat: ComponentCategory) -> u8 {
    match cat {
        ComponentCategory::Passives => 0,
        ComponentCategory::Sources => 1,
        ComponentCategory::Discretes => 2,
        ComponentCategory::Transistors => 3,
        ComponentCategory::LogicGates => 4,
        ComponentCategory::IntegratedCircuits => 5,
        ComponentCategory::AnalogICs => 6,
        ComponentCategory::Indicators => 7,
        ComponentCategory::Sensors => 8,
        ComponentCategory::TopologicalMetamaterials => 9,
        ComponentCategory::PortHamiltonian => 10,
    }
}

/// Writes an individual component to a binary buffer.
pub fn write_component(comp: &SchematicComponent, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(comp.id as u32).to_le_bytes());
    let kind_disc = component_kind_to_discriminant(comp.kind);
    buf.extend_from_slice(&kind_disc.to_le_bytes());
    let cat_disc = component_category_to_discriminant(comp.kind.category());
    buf.push(cat_disc);
    buf.extend_from_slice(&comp.pos.x.to_le_bytes());
    buf.extend_from_slice(&comp.pos.y.to_le_bytes());
    let rot_byte = (comp.rotation & 0x7F) | if comp.mirrored { 0x80 } else { 0 };
    buf.push(rot_byte);
    let val_bytes = comp.value_str.as_bytes();
    let val_len = val_bytes.len().min(u16::MAX as usize) as u16;
    buf.extend_from_slice(&val_len.to_le_bytes());
    buf.extend_from_slice(&val_bytes[..val_len as usize]);

    let mut props_to_write = comp.properties.clone();
    if (comp.scale - 1.0).abs() > 1e-4 && !props_to_write.iter().any(|(k, _)| k == "scale") {
        props_to_write.push(("scale".to_string(), format!("{:.2}", comp.scale)));
    }

    let prop_count = props_to_write.len().min(u16::MAX as usize) as u16;
    buf.extend_from_slice(&prop_count.to_le_bytes());
    for (k, v) in props_to_write.iter().take(prop_count as usize) {
        let k_bytes = k.as_bytes();
        let k_len = k_bytes.len().min(u16::MAX as usize) as u16;
        buf.extend_from_slice(&k_len.to_le_bytes());
        buf.extend_from_slice(&k_bytes[..k_len as usize]);

        let v_bytes = v.as_bytes();
        let v_len = v_bytes.len().min(u16::MAX as usize) as u16;
        buf.extend_from_slice(&v_len.to_le_bytes());
        buf.extend_from_slice(&v_bytes[..v_len as usize]);
    }
}

/// Writes an individual wire to a binary buffer.
pub fn write_wire(wire: &SchematicWire, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(wire.id as u32).to_le_bytes());
    let start = wire.start_point();
    let end = wire.end_point();
    buf.extend_from_slice(&start.x.to_le_bytes());
    buf.extend_from_slice(&start.y.to_le_bytes());
    buf.extend_from_slice(&end.x.to_le_bytes());
    buf.extend_from_slice(&end.y.to_le_bytes());

    let net_bytes = wire.net_name.as_deref().unwrap_or("").as_bytes();
    let net_len = net_bytes.len().min(u16::MAX as usize) as u16;
    buf.extend_from_slice(&net_len.to_le_bytes());
    buf.extend_from_slice(&net_bytes[..net_len as usize]);

    let bw = wire.bit_width.clamp(1, u16::MAX as u32) as u16;
    buf.extend_from_slice(&bw.to_le_bytes());
}

/// Reads an individual component from a binary byte slice at the given cursor.
pub fn read_component(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<SchematicComponent, BinaryFormatError> {
    if *cursor + 4 + 2 + 1 + 4 + 4 + 1 + 2 > bytes.len() {
        return Err(BinaryFormatError::TruncatedData(
            "component record truncated".to_string(),
        ));
    }
    let id = u32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]) as usize;
    *cursor += 4;
    let kind_disc = u16::from_le_bytes([bytes[*cursor], bytes[*cursor + 1]]);
    *cursor += 2;
    let kind = component_kind_from_discriminant(kind_disc)?;
    let _cat_disc = bytes[*cursor];
    *cursor += 1;
    let pos_x = f32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]);
    *cursor += 4;
    let pos_y = f32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]);
    *cursor += 4;
    let rot_byte = bytes[*cursor];
    *cursor += 1;
    let rotation = rot_byte & 0x7F;
    let mut mirrored = (rot_byte & 0x80) != 0;
    let val_len = u16::from_le_bytes([bytes[*cursor], bytes[*cursor + 1]]) as usize;
    *cursor += 2;
    if *cursor + val_len > bytes.len() {
        return Err(BinaryFormatError::TruncatedData(
            "component value string truncated".to_string(),
        ));
    }
    let value_str = std::str::from_utf8(&bytes[*cursor..*cursor + val_len])
        .map_err(|e| BinaryFormatError::Utf8Error(e.to_string()))?
        .to_string();
    *cursor += val_len;

    if *cursor + 2 > bytes.len() {
        return Err(BinaryFormatError::TruncatedData(
            "component properties count truncated".to_string(),
        ));
    }
    let prop_count = u16::from_le_bytes([bytes[*cursor], bytes[*cursor + 1]]) as usize;
    *cursor += 2;

    let mut name = None;
    let mut model_name = None;
    let mut properties = Vec::with_capacity(prop_count);
    for _ in 0..prop_count {
        if *cursor + 2 > bytes.len() {
            return Err(BinaryFormatError::TruncatedData(
                "property key length truncated".to_string(),
            ));
        }
        let k_len = u16::from_le_bytes([bytes[*cursor], bytes[*cursor + 1]]) as usize;
        *cursor += 2;
        if *cursor + k_len > bytes.len() {
            return Err(BinaryFormatError::TruncatedData(
                "property key bytes truncated".to_string(),
            ));
        }
        let key = std::str::from_utf8(&bytes[*cursor..*cursor + k_len])
            .map_err(|e| BinaryFormatError::Utf8Error(e.to_string()))?
            .to_string();
        *cursor += k_len;

        if *cursor + 2 > bytes.len() {
            return Err(BinaryFormatError::TruncatedData(
                "property value length truncated".to_string(),
            ));
        }
        let v_len = u16::from_le_bytes([bytes[*cursor], bytes[*cursor + 1]]) as usize;
        *cursor += 2;
        if *cursor + v_len > bytes.len() {
            return Err(BinaryFormatError::TruncatedData(
                "property value bytes truncated".to_string(),
            ));
        }
        let val = std::str::from_utf8(&bytes[*cursor..*cursor + v_len])
            .map_err(|e| BinaryFormatError::Utf8Error(e.to_string()))?
            .to_string();
        *cursor += v_len;

        if key == "name" {
            name = Some(val.clone());
        } else if key == "model_name" {
            model_name = Some(val.clone());
        }
        properties.push((key, val));
    }

    let name = name.unwrap_or_else(|| {
        use std::fmt::Write;
        let mut s = String::with_capacity(kind.prefix().len() + 8);
        s.push_str(kind.prefix());
        let _ = write!(s, "{}", id);
        s
    });

    if properties.iter().any(|(k, v)| k == "mirrored" && v == "true") {
        mirrored = true;
    }

    let scale = properties
        .iter()
        .find(|(k, _)| k == "scale")
        .and_then(|(_, v)| v.parse::<f32>().ok())
        .unwrap_or(1.0);

    Ok(SchematicComponent {
        id,
        name,
        kind,
        pos: Pos2::new(pos_x, pos_y),
        rotation,
        mirrored,
        scale,
        value_str,
        model_name,
        properties,
    })
}

/// Reads an individual wire from a binary byte slice at the given cursor.
pub fn read_wire(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<SchematicWire, BinaryFormatError> {
    if *cursor + 4 + 4 + 4 + 4 + 4 + 2 > bytes.len() {
        return Err(BinaryFormatError::TruncatedData(
            "wire record truncated".to_string(),
        ));
    }
    let id = u32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]) as usize;
    *cursor += 4;
    let start_x = f32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]);
    *cursor += 4;
    let start_y = f32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]);
    *cursor += 4;
    let end_x = f32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]);
    *cursor += 4;
    let end_y = f32::from_le_bytes([
        bytes[*cursor],
        bytes[*cursor + 1],
        bytes[*cursor + 2],
        bytes[*cursor + 3],
    ]);
    *cursor += 4;
    let net_len = u16::from_le_bytes([bytes[*cursor], bytes[*cursor + 1]]) as usize;
    *cursor += 2;
    if *cursor + net_len > bytes.len() {
        return Err(BinaryFormatError::TruncatedData(
            "wire net name truncated".to_string(),
        ));
    }
    let net_name_str = std::str::from_utf8(&bytes[*cursor..*cursor + net_len])
        .map_err(|e| BinaryFormatError::Utf8Error(e.to_string()))?;
    *cursor += net_len;

    let net_name = if net_name_str.is_empty() {
        None
    } else {
        Some(net_name_str.to_string())
    };

    let start = Pos2::new(start_x, start_y);
    let end = Pos2::new(end_x, end_y);
    let bit_width = if *cursor + 2 <= bytes.len() {
        let bw = u16::from_le_bytes([bytes[*cursor], bytes[*cursor + 1]]) as u32;
        *cursor += 2;
        if bw == 0 { 1 } else { bw }
    } else {
        1
    };
    let mut wire = SchematicWire::manhattan_route_with_net(id, start, end, net_name);
    wire.bit_width = bit_width;
    Ok(wire)
}

/// Serializes project state into ultra-compact binary format (.phn).
pub fn serialize_project(
    title: &str,
    components: &[SchematicComponent],
    wires: &[SchematicWire],
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(128 + components.len() * 48 + wires.len() * 32);

    // 1. Header (24 bytes):
    buf.extend_from_slice(&PHONON_MAGIC);
    buf.extend_from_slice(&CURRENT_VERSION.to_le_bytes());
    let flags: u16 = 0;
    buf.extend_from_slice(&flags.to_le_bytes());
    let comp_count = components.len() as u32;
    buf.extend_from_slice(&comp_count.to_le_bytes());
    let wire_count = wires.len() as u32;
    buf.extend_from_slice(&wire_count.to_le_bytes());
    let title_bytes = title.as_bytes();
    let title_len = title_bytes.len() as u32;
    buf.extend_from_slice(&title_len.to_le_bytes());

    // 2. Title bytes:
    buf.extend_from_slice(title_bytes);

    // 3. Components:
    for comp in components {
        write_component(comp, &mut buf);
    }

    // 4. Wires:
    for wire in wires {
        write_wire(wire, &mut buf);
    }

    // 5. Checksum Footer (4 bytes LE Adler-32):
    let checksum = compute_adler32(&buf);
    buf.extend_from_slice(&checksum.to_le_bytes());

    buf
}

/// Deserializes a project from an ultra-compact binary byte slice (.phn).
pub fn deserialize_project(bytes: &[u8]) -> Result<DeserializedProject, BinaryFormatError> {
    if bytes.len() < 28 {
        return Err(BinaryFormatError::TruncatedData(format!(
            "expected at least 28 bytes for header and footer, got {}",
            bytes.len()
        )));
    }

    if &bytes[0..8] != &PHONON_MAGIC {
        return Err(BinaryFormatError::InvalidMagic);
    }

    let version = u16::from_le_bytes([bytes[8], bytes[9]]);
    if version != CURRENT_VERSION {
        return Err(BinaryFormatError::UnsupportedVersion(version));
    }

    let payload_len = bytes.len() - 4;
    let expected_checksum = u32::from_le_bytes([
        bytes[payload_len],
        bytes[payload_len + 1],
        bytes[payload_len + 2],
        bytes[payload_len + 3],
    ]);
    let computed_checksum = compute_adler32(&bytes[..payload_len]);
    if computed_checksum != expected_checksum {
        return Err(BinaryFormatError::CorruptChecksum);
    }

    let _flags = u16::from_le_bytes([bytes[10], bytes[11]]);
    let comp_count = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    let wire_count = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as usize;
    let title_len = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as usize;

    let mut cursor = 24;

    if cursor + title_len > payload_len {
        return Err(BinaryFormatError::TruncatedData(
            "title string truncated".to_string(),
        ));
    }
    let title_str = std::str::from_utf8(&bytes[cursor..cursor + title_len])
        .map_err(|e| BinaryFormatError::Utf8Error(e.to_string()))?
        .to_string();
    cursor += title_len;

    let payload = &bytes[..payload_len];
    let mut components = Vec::with_capacity(comp_count);
    for _ in 0..comp_count {
        components.push(read_component(payload, &mut cursor)?);
    }

    let mut wires = Vec::with_capacity(wire_count);
    for _ in 0..wire_count {
        wires.push(read_wire(payload, &mut cursor)?);
    }

    Ok(DeserializedProject {
        title: title_str,
        components,
        wires,
    })
}

/// Serializes and writes a project directly to a file on disk.
pub fn save_project_to_file<P: AsRef<Path>>(
    path: P,
    title: &str,
    components: &[SchematicComponent],
    wires: &[SchematicWire],
) -> Result<(), BinaryFormatError> {
    let bytes = serialize_project(title, components, wires);
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Reads and deserializes a project from a file on disk.
pub fn load_project_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<DeserializedProject, BinaryFormatError> {
    let bytes = std::fs::read(path)?;
    deserialize_project(&bytes)
}
