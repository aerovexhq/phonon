//! Neuromorphic computing, non-volatile memristors, and spiking neural networks.
//!
//! Submodules:
//! - [`rram`]: Filamentary Resistive RAM (RRAM / ReRAM) with oxygen vacancy migration dynamics.
//! - [`pcm`]: Phase-Change Memory (PCM / GST) with JMAK crystallization, melting, and OTS.
//! - [`fefet`]: Ferroelectric Field-Effect Transistor (FeFET / HZO) with Landau-Khalatnikov dynamics.
//! - [`crossbar`]: $M \times N$ memristive synaptic crossbar array with parasitic wire resistance and VMM.
//! - [`neuron`]: Leaky Integrate-and-Fire (LIF) and Adaptive Exponential (AdEx) biological spiking neurons.
//! - [`stdp`]: Spike-Timing-Dependent Plasticity (STDP) Hebbian synaptic learning rules.

pub mod crossbar;
pub mod fefet;
pub mod neuron;
pub mod pcm;
pub mod rram;
pub mod stdp;

pub use crossbar::{CrossbarCellType, MemristiveCrossbarModel};
pub use fefet::FerroelectricFetModel;
pub use neuron::{NeuronState, SpikingNeuronModel};
pub use pcm::PhaseChangeMemoryModel;
pub use rram::FilamentaryRramModel;
pub use stdp::SpikeTimingPlasticityModel;
