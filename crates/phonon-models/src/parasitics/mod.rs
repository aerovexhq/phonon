//! Physical passive component parasitics (ESR, ESL, dielectric loss, inductor core saturation).

pub mod capacitor_parasitic;
pub mod inductor_saturation;

pub use capacitor_parasitic::RealCapacitorModel;
pub use inductor_saturation::RealInductorModel;
