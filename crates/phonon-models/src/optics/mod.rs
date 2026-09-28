//! Optical Perception, CMOS Active Pixel Sensors & Camera Models
//!
//! Provides microscopic CMOS photodiode physics, quantum efficiency, noise sources,
//! camera optics, and non-linear Brown-Conrady lens distortion.

pub mod camera;
pub mod cmos_aps;

pub use camera::{CameraIntrinsics, OpticalCamera};
pub use cmos_aps::{
    silicon_quantum_efficiency, transduce_cmos_pixel, CmosPixelConfig, PixelOutput, ShutterType,
    SILICON_BANDGAP_JOULES, SILICON_CUTOFF_WAVELENGTH_NM,
};
