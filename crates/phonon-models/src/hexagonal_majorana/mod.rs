//! Hexagonal superconducting nanowire arrays, tri-junction geometries,
//! non-Abelian Majorana braiding protocols, and parity readout models.

pub mod braiding_protocol;
pub mod hexagonal_array;

pub use braiding_protocol::{
    AliceaBraidStage, AliceaTriJunctionBraiding, GateRampProfile, TopologicalQubitRegister,
};
pub use hexagonal_array::{
    HexagonalSuperconductingArray, InPlaneMagneticField, MajoranaMaterialParams, MajoranaZeroMode,
    NanowireSegment, TriJunction,
};
