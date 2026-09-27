//! Electromagnetic Wave Propagation, 3D Vector Electrodynamics & Geodetic Space RF Environments.

pub mod antenna;
pub mod channel_fading;
pub mod coordinates;
pub mod dielectric;
pub mod modulation;
pub mod noise;
pub mod rf_emitter;
pub mod space_channel;
pub mod vector_wave;
pub mod wifi_mac;

pub use antenna::{AntennaGeometry, PhysicalAntenna, VACUUM_IMPEDANCE, VACUUM_PERMEABILITY};
pub use channel_fading::{ChannelProfile, ChannelRng, ChannelTap, FadingChannel};
pub use coordinates::{
    EarthHorizon, EcefCoord, EnuCoord, GeodeticCoord, MEAN_EARTH_RADIUS_METERS, STANDARD_K_FACTOR,
    WGS84_A_METERS, WGS84_B_METERS, WGS84_E_SQ, WGS84_FLATTENING,
};
pub use dielectric::{
    DielectricWall, FresnelCoefficients, KnifeEdgeObstacle, RayHit, RfDielectricMaterial,
};
pub use modulation::{erfc, q_function, ModulationScheme, OfdmConfig};
pub use noise::{
    RfNoiseModel, COSMIC_MICROWAVE_BACKGROUND_KELVIN, SOLAR_DISK_DIAMETER_DEG,
    STANDARD_NOISE_TEMP_KELVIN,
};
pub use rf_emitter::{
    AmplifierClass, DiscreteTransmitter, OscillatorType, RfPowerAmplifier, BOLTZMANN_CONSTANT,
    REFERENCE_TEMP_K,
};
pub use space_channel::{DopplerResult, SpaceNode, IONO_DISPERSION_CONSTANT, ONE_TECU};
pub use vector_wave::{
    ComplexField3D, EmWaveSource, Polarization, Vector3D, INTRINSIC_IMPEDANCE_VACUUM,
};
pub use wifi_mac::{
    compute_crc32, CsmaCaConfig, CsmaCaStation, FrameType, MacAddress, MacFrame, StationState,
    WifiPhyStandard,
};
