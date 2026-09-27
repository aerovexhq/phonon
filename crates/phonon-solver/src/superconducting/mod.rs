//! Cryogenic Hybrid Superconducting-Photonic Coprocessor, SOEN neural networks,
//! and real-time surface code quantum error correction (QEC) decoding.
//!
//! Submodules:
//! - [`soen_network`]: Superconducting Optoelectronic Spiking Neural Network and optical interconnect.
//! - [`qec_decoder`]: Real-time cryogenic surface code syndrome decoder ($d=3, d=5$).
//! - [`coprocessor_benchmark`]: Multi-threaded Rayon benchmark comparing cryogenic coprocessor against 3nm CMOS.

pub mod coprocessor_benchmark;
pub mod qec_decoder;
pub mod soen_network;

pub use coprocessor_benchmark::{
    ClassicalCmosBaseline, CoprocessorComparisonReport, HybridCoprocessorBenchmarkRunner,
};
pub use qec_decoder::{CryoDecoderEngine, CryoQecDecoder, DecodingResult, QecCorrection};
pub use soen_network::{OpticalInterconnect, SoenNetwork, SoenSpikeRecord, TravelingOpticalPacket};
