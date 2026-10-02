#![deny(unsafe_code)]

//! Port-Hamiltonian Audio-Acoustic Multi-Physics Engine & Symplectic MNA Stamp Library.

pub mod audio_engine;
pub mod dirac;
pub mod mna_stamps;
pub mod vocal_fold;
pub mod webster_horn;

pub use audio_engine::*;
pub use dirac::*;
pub use mna_stamps::*;
pub use vocal_fold::*;
pub use webster_horn::*;
