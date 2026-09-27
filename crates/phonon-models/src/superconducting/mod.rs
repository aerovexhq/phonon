//! Superconducting electronics, Josephson junctions, Single Flux Quantum (SFQ) logic,
//! Superconducting Optoelectronic Neurons (SOEN), and cryogenic QEC interfaces.
//!
//! Submodules:
//! - [`material`]: Superconductor material properties, BCS gap dynamics, and Ambegaokar-Baratoff $I_c R_n$.
//! - [`rcsj`]: Resistively and Capacitively Shunted Junction (RCSJ) macro-model and dynamic companion stamping.
//! - [`sfq`]: Single Flux Quantum soliton pulses ($\int V dt = \Phi_0$), JTL stages, and DC-SQUIDs.
//! - [`soen`]: Superconducting Optoelectronic Neurons, SNSPD detectors, and cryogenic optical emitters.
//! - [`qec_interface`]: RSFQ logic primitives, optoelectronic transducers, and surface code syndrome structures.

pub mod material;
pub mod qec_interface;
pub mod rcsj;
pub mod sfq;
pub mod soen;

pub use material::SuperconductorMaterial;
pub use qec_interface::{
    OptoToRsfqTransducer, PauliCorrection, RsfqAnd, RsfqDff, RsfqInverter, RsfqJtl,
    RsfqToOptoDriver, SurfaceCodeGeometry, SyndromePacket,
};
pub use rcsj::{JosephsonRcsjModel, RcsjCompanionStamp};
pub use sfq::{integrate_voltage_time, verify_flux_quantization, DcSquidModel, JtlStage, SfqPulse};
pub use soen::{CryoOpticalEmitter, SnspdModel, SoenMetrics, SoenNeuron, SuperconductingFluxLoop};
