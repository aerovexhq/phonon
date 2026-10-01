#![deny(unsafe_code)]

//! SPICE model parameter extraction wizard, genetic algorithm curve-fitting engine, and BSIM4 parameter tuning.

pub mod curve_data;
pub mod model_deck;
pub mod optimizer;

pub use curve_data::{ExtractionError, MeasuredCurve, MeasurementPoint};
pub use model_deck::{generate_bsim4_model_deck, validate_bsim4_model_deck};
pub use optimizer::{genome_to_params, params_to_genome, Bsim4TargetParams, FittingResult, GaOptimizer};
