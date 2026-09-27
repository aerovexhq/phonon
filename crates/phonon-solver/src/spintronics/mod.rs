//! Nanomagnetic logic solver modules, geometric synthesis, and comparative benchmarking.
//!
//! Modules:
//! - `micromagnetic_array`: 2D spatial array of interacting nanomagnets with multiphase clocking.
//! - `nml_synthesis`: Autonomous geometric synthesizer for NML logic gates.
//! - `nml_benchmark`: Multi-threaded Rayon comparative benchmark engine vs 3nm CMOS.

pub mod micromagnetic_array;
pub mod nml_benchmark;
pub mod nml_synthesis;

pub use micromagnetic_array::{ClockPhase, MicromagneticArray};
pub use nml_benchmark::{Cmos3nmReference, NmlBenchmarkRunner, NmlComparisonReport};
pub use nml_synthesis::{NmlLogicSynthesizer, SynthesizedNmlLogic, TargetNmlFunction};
