//! Radiation effects, Single-Event Effects (SEE), Total Ionizing Dose (TID),
//! Displacement Damage Dose (DDD), Single-Event Latchup (SEL), and Radiation-Hardened-by-Design (RHBD).

pub mod ddd;
pub mod latchup;
pub mod let_track;
pub mod rhbd;
pub mod tid;

pub use ddd::{DisplacementDamageModel, RAD_SI_TO_MEV_PER_G};
pub use latchup::{LatchupEvaluation, ParasiticThyristorModel};
pub use let_track::{HeavyIonStrikeModel, E_EH_SILICON_JOULES};
pub use rhbd::{DiceCell, StandardSramCell, StrikeOutcome, TmrVoter};
pub use tid::{TotalIonizingDoseModel, DEFAULT_E_EH_OXIDE_EV};
