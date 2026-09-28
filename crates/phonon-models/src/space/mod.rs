//! Spacecraft GNC, Orbital Mechanics & Star Tracker Co-Simulation.

pub mod attitude;
pub mod orbit;
pub mod star_tracker;

pub use attitude::{
    earth_geomagnetic_field, InertiaTensor, MagneticTorquerSystem, ReactionWheel,
    ReactionWheelCluster,
};
pub use orbit::{
    KeplerianElements, OrbitalPerturbationSolver, SpacecraftPhysicalProperties, J2_EARTH, J3_EARTH,
    J4_EARTH, MU_EARTH, MU_MOON, MU_SUN, R_EARTH,
};
pub use star_tracker::{CatalogStar, StarTrackerCamera, StarTrackerSystem};
