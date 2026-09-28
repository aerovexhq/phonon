//! Nanomagnetic logic solver modules, geometric synthesis, and comparative benchmarking.
//!
//! Modules:
//! - `micromagnetic_array`: 2D spatial array of interacting nanomagnets with multiphase clocking.
//! - `nml_synthesis`: Autonomous geometric synthesizer for NML logic gates.
//! - `nml_benchmark`: Multi-threaded Rayon comparative benchmark engine vs 3nm CMOS.
//! - `lindblad_solver`: Lindbladian open-quantum-system master equation solver with detailed balance.
//! - `molecular_synthesis`: Autonomous synthesizer for molecular spintronic memory arrays.
//! - `molecular_benchmark`: Multi-threaded Rayon benchmark comparing Molecular Spintronics vs MTJ and 3nm CMOS.

pub mod lindblad_solver;
pub mod micromagnetic_array;
pub mod molecular_benchmark;
pub mod molecular_synthesis;
pub mod nml_benchmark;
pub mod nml_synthesis;

pub use lindblad_solver::{LindbladConfig, LindbladMasterSolver, LindbladTrajectory};
pub use micromagnetic_array::{ClockPhase, MicromagneticArray};
pub use molecular_benchmark::{
    Cmos3nmGaaReference, MolecularBenchmarkRunner, MolecularComparisonReport, MtjReference,
};
pub use molecular_synthesis::{
    MolecularMemoryArray, MolecularSpintronicSynthesizer, MolecularSynthesisTarget,
    SynthesizedMolecularCell,
};
pub use nml_benchmark::{Cmos3nmReference, NmlBenchmarkRunner, NmlComparisonReport};
pub use nml_synthesis::{NmlLogicSynthesizer, SynthesizedNmlLogic, TargetNmlFunction};
