//! Machine-learning accelerated neural surrogate metamodeling and Polynomial Chaos Expansion (PCE).

pub mod mlp;
pub mod pce;
pub mod surrogate_companion;
pub mod trainer;

pub use mlp::{ActivationFunction, DenseLayer, MultilayerPerceptron};
pub use pce::{eval_legendre, PceTerm, PolynomialChaosExpansion};
pub use surrogate_companion::{NeuralSurrogateCompanion, SurrogateDeviceType};
pub use trainer::{fit_pce_surrogate, train_mlp_surrogate, MlpTrainingConfig};
