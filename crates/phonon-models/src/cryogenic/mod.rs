//! Cryogenic semiconductor physics down to 4 Kelvin.
//!
//! Submodules:
//! - [`fermi_dirac`]: Fermi-Dirac integral $F_{1/2}(\eta)$ and Nilsson analytical inverse.
//! - [`freezeout`]: Incomplete dopant freeze-out and Poole-Frenkel electric-field ionization.
//! - [`cryo_mobility`]: Brooks-Herring, Erginsoy neutral impurity, and acoustic phonon scattering.
//! - [`cryo_mosfet`]: Cryo-CMOS compact model with cryogenic subthreshold swing and $V_{th}(T)$ shift.

pub mod cryo_mobility;
pub mod cryo_mosfet;
pub mod fermi_dirac;
pub mod freezeout;

pub use cryo_mobility::CryogenicMobilityModel;
pub use cryo_mosfet::{CryoMosfetModel, CryoMosfetOutput};
pub use fermi_dirac::{fermi_dirac_half, inverse_fermi_dirac_half};
pub use freezeout::CryogenicFreezeoutModel;
