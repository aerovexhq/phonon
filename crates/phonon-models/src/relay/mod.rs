//! Electrochemical metallization, conductive bridging, and nanoelectromechanical atomic relay models.
//!
//! Provides:
//! - Electrochemical Metallization Cells (ECM) and Conductive Bridging RAM (`ecm_cell`).
//! - 3-Terminal Nanoelectromechanical Atomic Relays (`atomic_relay`).
//! - Zero-leakage logic primitives and full adders (`relay_primitives`).

pub mod atomic_relay;
pub mod ecm_cell;
pub mod relay_primitives;

pub use atomic_relay::{
    AtomicRelayModel, AtomicRelayParameters, AtomicRelayState, RelayContactState,
};
pub use ecm_cell::{
    EcmCellModel, EcmCellParameters, EcmCellState, EcmConductionState, EcmSwitchingMode, G_0, R_0,
};
pub use relay_primitives::{
    MultiBitRelayAdder, RelayFullAdderCell, RelayLogicGate, RelaySwitchType,
};
