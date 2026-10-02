#![deny(unsafe_code)]

//! Interactive Transient Audio DSP Synthesizer & Soundcard Driver.
//!
//! Submodules:
//! - ring_buffer: High-throughput circular FIFO audio buffer with underrun detection.
//! - vowel_morpher: Cardinal vowel vocal tract area function interpolation.
//! - plosive_engine: Articulatory occlusion and release turbulence burst aerodynamics.
//! - driver: Master soundcard audio driver integrating Port-Hamiltonian physics and soft limiting.

pub mod driver;
pub mod plosive_engine;
pub mod ring_buffer;
pub mod vowel_morpher;

pub use driver::*;
pub use plosive_engine::*;
pub use ring_buffer::*;
pub use vowel_morpher::*;
