//! Molecular spintronics, single-molecule magnetism (SMM), and spin-torque nano-oscillators (STNO).

pub mod giant_spin;
pub mod negf_molecular;
pub mod phase_locking;
pub mod spin_torque_oscillator;

pub use giant_spin::{GiantSpinParams, BOHR_MAGNETON, BOLTZMANN_K, HBAR};
pub use negf_molecular::NegfMolecularJunctionParams;
pub use phase_locking::InjectionLockingParams;
pub use spin_torque_oscillator::StnoParams;
