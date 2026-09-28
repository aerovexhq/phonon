//! Physics-Coupled Sensors & Multi-Axis Inertial Transducers
//!
//! Provides models for tactile pressure sensors, contact manifolds,
//! and 6-DOF / 9-DOF MEMS IMUs with Allan variance noise processes.

pub mod flight_dynamics;
pub mod imu;
pub mod nv_center;
pub mod tactile;

pub use flight_dynamics::{
    compute_ground_effect_factor, AirframeConfig, BarometerSensor, DrydenWindModel,
    FlightDynamicsEngine, FlightDynamicsState, GpsFixType, GpsMeasurement, InertiaTensor3D,
    LidarRangefinder, RotorConfig, BAROMETRIC_EXPONENT, DRY_AIR_MOLAR_MASS_KG_PER_MOL,
    STANDARD_SEA_LEVEL_PRESSURE_PA, STANDARD_SEA_LEVEL_TEMP_K, STANDARD_TEMP_LAPSE_RATE_K_PER_M,
    UNIVERSAL_GAS_CONSTANT,
};
pub use imu::{
    sample_imu, AllanNoiseConfig, ImuConfig, ImuMeasurement, ImuState, Quaternion,
    STANDARD_GRAVITY_M_S2,
};
pub use nv_center::{
    reconstruct_vector_magnetic_field, NitrogenIsotope, NvCenter, NvOrientation, OdmrConfig,
    OdmrSpectrum, BOHR_MAGNETON_JOULES, N14_HYPERFINE_PARALLEL_HZ, N14_HYPERFINE_PERP_HZ,
    N14_QUADRUPOLE_SPLITTING_HZ, N15_HYPERFINE_PARALLEL_HZ, N15_HYPERFINE_PERP_HZ,
    NV_D_TEMP_COEFFICIENT_HZ_PER_K, NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T, NV_ELECTRON_G_FACTOR,
    NV_ZERO_FIELD_SPLITTING_D_HZ,
};
pub use tactile::{
    CapacitiveSensorConfig, CollisionContactInput, PiezoresistiveSensorConfig, TactileMatrixArray,
};
