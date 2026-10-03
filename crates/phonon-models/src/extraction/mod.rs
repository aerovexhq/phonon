#![deny(unsafe_code)]

//! SPICE model parameter extraction wizard, genetic algorithm curve-fitting engine, and compact model parameter tuning.

pub mod bjt;
pub mod curve_data;
pub mod ekv;
pub mod model_deck;
pub mod optimizer;
pub mod polisher;

pub use bjt::{
    bjt_genome_to_params, bjt_params_to_genome, BjtBounds, BjtFittingResult, BjtOptimizer,
    BjtTargetParams,
};
pub use curve_data::{ExtractionError, MeasuredCurve, MeasurementPoint};
pub use ekv::{
    ekv_genome_to_params, ekv_params_to_genome, EkvBounds, EkvFittingResult, EkvOptimizer,
    EkvTargetParams,
};
pub use model_deck::{
    generate_bjt_model_deck, generate_bsim4_model_deck, generate_ekv_model_deck,
    validate_bjt_model_deck, validate_bsim4_model_deck, validate_ekv_model_deck,
};
pub use optimizer::{
    genome_to_params, params_to_genome, Bsim4Bounds, Bsim4FittingResult, Bsim4TargetParams,
    FittingResult, GaOptimizer,
};
pub use polisher::{
    polish_model_parameters, polish_parameters, GenericFittingResult, PolishableModel,
};
