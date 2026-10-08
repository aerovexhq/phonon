#![deny(unsafe_code)]

//! Compact, scalable, and fully deterministic binary history format for Phonon Studio.
//!
//! Provides extensible opcode-tagged binary serialization of undo/redo action streams
//! with configurable in-memory and on-disk depth limits.

use super::history::{CanvasCommand, HistoryStack};
use super::net_label::{NetLabel, NetLabelOrientation};
use crate::schematic::binary_format::{
    read_component, read_wire, write_component, write_wire, BinaryFormatError,
};
use egui::Pos2;
use std::fmt;

/// Magic 4-byte header identifying Phonon Binary History streams ("PHNH").
pub const HISTORY_MAGIC: &[u8; 4] = b"PHNH";

/// Active binary history format specification version.
pub const HISTORY_VERSION: u16 = 1;

/// Numeric opcode uniquely identifying every reversible canvas mutation action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ActionOpcode {
    AddComponent = 0x01,
    DeleteComponent = 0x02,
    MoveComponent = 0x03,
    RotateComponent = 0x04,
    ModifyComponentValue = 0x05,
    AddWire = 0x06,
    DeleteWire = 0x07,
    ClearAll = 0x08,
    Batch = 0x09,
    MirrorComponent = 0x0A,
    AddNetLabel = 0x0B,
    DeleteNetLabel = 0x0C,
    MoveNetLabel = 0x0D,
    ScaleComponent = 0x0E,
}

impl ActionOpcode {
    pub fn from_u8(val: u8) -> Result<Self, BinaryHistoryError> {
        match val {
            0x01 => Ok(Self::AddComponent),
            0x02 => Ok(Self::DeleteComponent),
            0x03 => Ok(Self::MoveComponent),
            0x04 => Ok(Self::RotateComponent),
            0x05 => Ok(Self::ModifyComponentValue),
            0x06 => Ok(Self::AddWire),
            0x07 => Ok(Self::DeleteWire),
            0x08 => Ok(Self::ClearAll),
            0x09 => Ok(Self::Batch),
            0x0A => Ok(Self::MirrorComponent),
            0x0B => Ok(Self::AddNetLabel),
            0x0C => Ok(Self::DeleteNetLabel),
            0x0D => Ok(Self::MoveNetLabel),
            0x0E => Ok(Self::ScaleComponent),
            other => Err(BinaryHistoryError::UnknownOpcode(other)),
        }
    }
}

/// Errors occurring during binary history encoding or decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryHistoryError {
    InvalidMagic,
    UnsupportedVersion(u16),
    TruncatedData,
    ChecksumMismatch { expected: u32, computed: u32 },
    UnknownOpcode(u8),
    InvalidUtf8,
    FormatError(String),
}

impl fmt::Display for BinaryHistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "Invalid history magic bytes (expected 'PHNH')"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported history version {}", v),
            Self::TruncatedData => write!(f, "Unexpected end of binary history stream"),
            Self::ChecksumMismatch { expected, computed } => {
                write!(
                    f,
                    "History checksum mismatch: expected {:#010x}, got {:#010x}",
                    expected, computed
                )
            }
            Self::UnknownOpcode(op) => write!(f, "Unknown action opcode {:#04x}", op),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 string encoding in action payload"),
            Self::FormatError(msg) => write!(f, "History format error: {}", msg),
        }
    }
}

impl std::error::Error for BinaryHistoryError {}

impl From<BinaryFormatError> for BinaryHistoryError {
    fn from(err: BinaryFormatError) -> Self {
        Self::FormatError(err.to_string())
    }
}

/// Computes an Adler-32 checksum over the given payload slice.
pub fn adler32(data: &[u8]) -> u32 {
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

/// Serializes an individual CanvasCommand to a binary stream.
pub fn write_command(cmd: &CanvasCommand, out: &mut Vec<u8>) {
    match cmd {
        CanvasCommand::AddComponent(comp) => {
            out.push(ActionOpcode::AddComponent as u8);
            write_component(comp, out);
        }
        CanvasCommand::DeleteComponent(comp) => {
            out.push(ActionOpcode::DeleteComponent as u8);
            write_component(comp, out);
        }
        CanvasCommand::MoveComponent { id, from, to } => {
            out.push(ActionOpcode::MoveComponent as u8);
            out.extend_from_slice(&(*id as u64).to_le_bytes());
            out.extend_from_slice(&from.x.to_le_bytes());
            out.extend_from_slice(&from.y.to_le_bytes());
            out.extend_from_slice(&to.x.to_le_bytes());
            out.extend_from_slice(&to.y.to_le_bytes());
        }
        CanvasCommand::RotateComponent {
            id,
            from_rot,
            to_rot,
        } => {
            out.push(ActionOpcode::RotateComponent as u8);
            out.extend_from_slice(&(*id as u64).to_le_bytes());
            out.push(*from_rot);
            out.push(*to_rot);
        }
        CanvasCommand::MirrorComponent {
            id,
            from_mirrored,
            to_mirrored,
        } => {
            out.push(ActionOpcode::MirrorComponent as u8);
            out.extend_from_slice(&(*id as u64).to_le_bytes());
            out.push(if *from_mirrored { 1 } else { 0 });
            out.push(if *to_mirrored { 1 } else { 0 });
        }
        CanvasCommand::ScaleComponent {
            id,
            from_scale,
            to_scale,
        } => {
            out.push(ActionOpcode::ScaleComponent as u8);
            out.extend_from_slice(&(*id as u64).to_le_bytes());
            out.extend_from_slice(&from_scale.to_le_bytes());
            out.extend_from_slice(&to_scale.to_le_bytes());
        }
        CanvasCommand::ModifyComponentValue {
            id,
            old_val,
            new_val,
        } => {
            out.push(ActionOpcode::ModifyComponentValue as u8);
            out.extend_from_slice(&(*id as u64).to_le_bytes());
            let old_bytes = old_val.as_bytes();
            out.extend_from_slice(&(old_bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(old_bytes);
            let new_bytes = new_val.as_bytes();
            out.extend_from_slice(&(new_bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(new_bytes);
        }
        CanvasCommand::AddWire(wire) => {
            out.push(ActionOpcode::AddWire as u8);
            write_wire(wire, out);
        }
        CanvasCommand::DeleteWire(wire) => {
            out.push(ActionOpcode::DeleteWire as u8);
            write_wire(wire, out);
        }
        CanvasCommand::ClearAll { components, wires, .. } => {
            out.push(ActionOpcode::ClearAll as u8);
            out.extend_from_slice(&(components.len() as u32).to_le_bytes());
            for c in components {
                write_component(c, out);
            }
            out.extend_from_slice(&(wires.len() as u32).to_le_bytes());
            for w in wires {
                write_wire(w, out);
            }
        }
        CanvasCommand::Batch(sub_cmds) => {
            out.push(ActionOpcode::Batch as u8);
            out.extend_from_slice(&(sub_cmds.len() as u32).to_le_bytes());
            for sub in sub_cmds {
                write_command(sub, out);
            }
        }
        CanvasCommand::AddNetLabel(lbl) => {
            out.push(ActionOpcode::AddNetLabel as u8);
            write_net_label(lbl, out);
        }
        CanvasCommand::DeleteNetLabel(lbl) => {
            out.push(ActionOpcode::DeleteNetLabel as u8);
            write_net_label(lbl, out);
        }
        CanvasCommand::MoveNetLabel { id, from, to } => {
            out.push(ActionOpcode::MoveNetLabel as u8);
            out.extend_from_slice(&(*id as u64).to_le_bytes());
            out.extend_from_slice(&from.x.to_le_bytes());
            out.extend_from_slice(&from.y.to_le_bytes());
            out.extend_from_slice(&to.x.to_le_bytes());
            out.extend_from_slice(&to.y.to_le_bytes());
        }
    }
}

/// Reads a single CanvasCommand from the byte cursor.
pub fn read_command(data: &[u8], cursor: &mut usize) -> Result<CanvasCommand, BinaryHistoryError> {
    if *cursor >= data.len() {
        return Err(BinaryHistoryError::TruncatedData);
    }

    let op_byte = data[*cursor];
    *cursor += 1;
    let opcode = ActionOpcode::from_u8(op_byte)?;

    match opcode {
        ActionOpcode::AddComponent => {
            let comp = read_component(data, cursor)?;
            Ok(CanvasCommand::AddComponent(comp))
        }
        ActionOpcode::DeleteComponent => {
            let comp = read_component(data, cursor)?;
            Ok(CanvasCommand::DeleteComponent(comp))
        }
        ActionOpcode::MoveComponent => {
            if *cursor + 24 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let id = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
            *cursor += 8;
            let fx = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            let fy = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            let tx = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            let ty = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            Ok(CanvasCommand::MoveComponent {
                id,
                from: Pos2::new(fx, fy),
                to: Pos2::new(tx, ty),
            })
        }
        ActionOpcode::RotateComponent => {
            if *cursor + 10 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let id = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
            *cursor += 8;
            let from_rot = data[*cursor];
            *cursor += 1;
            let to_rot = data[*cursor];
            *cursor += 1;
            Ok(CanvasCommand::RotateComponent {
                id,
                from_rot,
                to_rot,
            })
        }
        ActionOpcode::MirrorComponent => {
            if *cursor + 10 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let id = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
            *cursor += 8;
            let from_mirrored = data[*cursor] != 0;
            *cursor += 1;
            let to_mirrored = data[*cursor] != 0;
            *cursor += 1;
            Ok(CanvasCommand::MirrorComponent {
                id,
                from_mirrored,
                to_mirrored,
            })
        }
        ActionOpcode::ScaleComponent => {
            if *cursor + 16 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let id = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
            *cursor += 8;
            let from_scale = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            let to_scale = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            Ok(CanvasCommand::ScaleComponent {
                id,
                from_scale,
                to_scale,
            })
        }
        ActionOpcode::ModifyComponentValue => {
            if *cursor + 8 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let id = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
            *cursor += 8;

            if *cursor + 4 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let old_len = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap()) as usize;
            *cursor += 4;
            if *cursor + old_len > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let old_val = std::str::from_utf8(&data[*cursor..*cursor + old_len])
                .map_err(|_| BinaryHistoryError::InvalidUtf8)?
                .to_string();
            *cursor += old_len;

            if *cursor + 4 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let new_len = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap()) as usize;
            *cursor += 4;
            if *cursor + new_len > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let new_val = std::str::from_utf8(&data[*cursor..*cursor + new_len])
                .map_err(|_| BinaryHistoryError::InvalidUtf8)?
                .to_string();
            *cursor += new_len;

            Ok(CanvasCommand::ModifyComponentValue {
                id,
                old_val,
                new_val,
            })
        }
        ActionOpcode::AddWire => {
            let wire = read_wire(data, cursor)?;
            Ok(CanvasCommand::AddWire(wire))
        }
        ActionOpcode::DeleteWire => {
            let wire = read_wire(data, cursor)?;
            Ok(CanvasCommand::DeleteWire(wire))
        }
        ActionOpcode::ClearAll => {
            if *cursor + 4 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let comp_count = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap()) as usize;
            *cursor += 4;
            let mut components = Vec::with_capacity(comp_count);
            for _ in 0..comp_count {
                components.push(read_component(data, cursor)?);
            }

            if *cursor + 4 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let wire_count = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap()) as usize;
            *cursor += 4;
            let mut wires = Vec::with_capacity(wire_count);
            for _ in 0..wire_count {
                wires.push(read_wire(data, cursor)?);
            }

            Ok(CanvasCommand::ClearAll {
                components,
                wires,
                net_labels: Vec::new(),
            })
        }
        ActionOpcode::Batch => {
            if *cursor + 4 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let sub_count = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap()) as usize;
            *cursor += 4;
            let mut sub_cmds = Vec::with_capacity(sub_count);
            for _ in 0..sub_count {
                sub_cmds.push(read_command(data, cursor)?);
            }
            Ok(CanvasCommand::Batch(sub_cmds))
        }
        ActionOpcode::AddNetLabel => {
            let lbl = read_net_label(data, cursor)?;
            Ok(CanvasCommand::AddNetLabel(lbl))
        }
        ActionOpcode::DeleteNetLabel => {
            let lbl = read_net_label(data, cursor)?;
            Ok(CanvasCommand::DeleteNetLabel(lbl))
        }
        ActionOpcode::MoveNetLabel => {
            if *cursor + 24 > data.len() {
                return Err(BinaryHistoryError::TruncatedData);
            }
            let id = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
            *cursor += 8;
            let fx = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            let fy = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            let tx = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            let ty = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            Ok(CanvasCommand::MoveNetLabel {
                id,
                from: Pos2::new(fx, fy),
                to: Pos2::new(tx, ty),
            })
        }
    }
}

fn write_net_label(lbl: &NetLabel, out: &mut Vec<u8>) {
    out.extend_from_slice(&(lbl.id as u64).to_le_bytes());
    let name_bytes = lbl.name.as_bytes();
    out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
    out.extend_from_slice(name_bytes);
    out.extend_from_slice(&lbl.pos.x.to_le_bytes());
    out.extend_from_slice(&lbl.pos.y.to_le_bytes());
    out.push(match lbl.orientation {
        NetLabelOrientation::East => 0,
        NetLabelOrientation::South => 1,
        NetLabelOrientation::West => 2,
        NetLabelOrientation::North => 3,
    });
}

fn read_net_label(data: &[u8], cursor: &mut usize) -> Result<NetLabel, BinaryHistoryError> {
    if *cursor + 8 > data.len() {
        return Err(BinaryHistoryError::TruncatedData);
    }
    let id = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
    *cursor += 8;

    if *cursor + 2 > data.len() {
        return Err(BinaryHistoryError::TruncatedData);
    }
    let name_len = u16::from_le_bytes(data[*cursor..*cursor + 2].try_into().unwrap()) as usize;
    *cursor += 2;

    if *cursor + name_len > data.len() {
        return Err(BinaryHistoryError::TruncatedData);
    }
    let name = std::str::from_utf8(&data[*cursor..*cursor + name_len])
        .map_err(|_| BinaryHistoryError::InvalidUtf8)?
        .to_string();
    *cursor += name_len;

    if *cursor + 9 > data.len() {
        return Err(BinaryHistoryError::TruncatedData);
    }
    let px = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
    *cursor += 4;
    let py = f32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
    *cursor += 4;
    let orient_byte = data[*cursor];
    *cursor += 1;
    let orientation = match orient_byte {
        0 => NetLabelOrientation::East,
        1 => NetLabelOrientation::South,
        2 => NetLabelOrientation::West,
        3 => NetLabelOrientation::North,
        _ => NetLabelOrientation::East,
    };

    Ok(NetLabel {
        id,
        name,
        pos: Pos2::new(px, py),
        orientation,
    })
}

/// Serializes a full `HistoryStack` into a compact binary byte array with optional on-disk depth truncation.
pub fn serialize_history(history: &HistoryStack, on_disk_limit: usize) -> Vec<u8> {
    let mut payload = Vec::with_capacity(256);

    // 1. Magic & Version
    payload.extend_from_slice(HISTORY_MAGIC);
    payload.extend_from_slice(&HISTORY_VERSION.to_le_bytes());

    // 2. Select commands within on_disk_limit
    let total_undo = history.undo_stack.len();
    let take_count = if on_disk_limit == 0 {
        0
    } else {
        total_undo.min(on_disk_limit)
    };
    let skip_count = total_undo.saturating_sub(take_count);
    let commands_to_save: &[CanvasCommand] = &history.undo_stack[skip_count..];

    // Compute adjusted clean index relative to truncated slice
    let adjusted_clean = if history.clean_index < skip_count {
        usize::MAX
    } else {
        history.clean_index - skip_count
    };

    // 3. Header fields
    payload.extend_from_slice(&(adjusted_clean as u32).to_le_bytes());
    payload.extend_from_slice(&(commands_to_save.len() as u32).to_le_bytes());

    // 4. Serialize commands
    for cmd in commands_to_save {
        write_command(cmd, &mut payload);
    }

    // 5. Append checksum
    let checksum = adler32(&payload);
    payload.extend_from_slice(&checksum.to_le_bytes());

    payload
}

/// Deserializes a binary byte array back into an active `HistoryStack`.
pub fn deserialize_history(
    data: &[u8],
    in_memory_limit: usize,
) -> Result<HistoryStack, BinaryHistoryError> {
    if data.len() < 14 {
        return Err(BinaryHistoryError::TruncatedData);
    }

    // 1. Check Magic
    if &data[0..4] != HISTORY_MAGIC {
        return Err(BinaryHistoryError::InvalidMagic);
    }

    // 2. Verify Checksum
    let payload_len = data.len() - 4;
    let computed_checksum = adler32(&data[..payload_len]);
    let expected_checksum =
        u32::from_le_bytes(data[payload_len..payload_len + 4].try_into().unwrap());
    if computed_checksum != expected_checksum {
        return Err(BinaryHistoryError::ChecksumMismatch {
            expected: expected_checksum,
            computed: computed_checksum,
        });
    }

    // 3. Check Version
    let mut cursor = 4;
    let version = u16::from_le_bytes(data[cursor..cursor + 2].try_into().unwrap());
    cursor += 2;
    if version != HISTORY_VERSION {
        return Err(BinaryHistoryError::UnsupportedVersion(version));
    }

    // 4. Header fields
    let clean_index_raw = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
    let clean_index = if clean_index_raw == u32::MAX {
        usize::MAX
    } else {
        clean_index_raw as usize
    };
    cursor += 4;
    let cmd_count = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap()) as usize;
    cursor += 4;

    // 5. Commands
    let mut undo_stack = Vec::with_capacity(cmd_count);
    for _ in 0..cmd_count {
        let cmd = read_command(data, &mut cursor)?;
        undo_stack.push(cmd);
    }

    let mut stack = HistoryStack::with_max_depth(in_memory_limit);
    stack.undo_stack = undo_stack;
    stack.clean_index = clean_index;

    Ok(stack)
}
