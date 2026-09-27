//! Photonic and optoelectronic models for Photonic Integrated Circuits (PIC).
//!
//! Submodules:
//! - [`waveguide`]: Integrated optical waveguides (SOI strip, SiN) with thermo-optic drift and dispersion.
//! - [`ring_resonator`]: Optical micro-ring resonators (MRR) with all-pass and add-drop configurations.
//! - [`modulator`]: Electro-optic modulators (Mach-Zehnder Interferometers and Electro-Absorption).
//! - [`laser_diode`]: Semiconductor laser diode single-mode rate equations and L-I characteristics.
//! - [`photodetector`]: High-speed PIN and APD photodetectors with transit-time/RC bandwidth and noise.
//! - [`telecom`]: Digital optical communications telemetry, eye diagrams, Q-factor, and BER.

pub mod laser_diode;
pub mod modulator;
pub mod photodetector;
pub mod ring_resonator;
pub mod telecom;
pub mod waveguide;

pub use laser_diode::{LaserDiodeModel, LaserDiodeState};
pub use modulator::{ElectroOpticModulatorModel, ModulatorType};
pub use photodetector::{PhotodetectorModel, PhotodetectorType};
pub use ring_resonator::{MicroRingResonatorModel, RingResonatorType};
pub use telecom::{EyeMetrics, EyeSample, TelecomAnalyzer};
pub use waveguide::OpticalWaveguideModel;
