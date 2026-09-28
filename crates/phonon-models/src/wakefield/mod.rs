//! Relativistic plasma wakefields, laser envelope dynamics, blowout bubble regimes,
//! and betatron synchrotron X-ray radiation.

pub mod bubble_regime;
pub mod laser_envelope;
pub mod plasma_channel;
pub mod synchrotron_radiation;

pub use bubble_regime::BubbleRegime;
pub use laser_envelope::LaserPulseParams;
pub use plasma_channel::PlasmaChannelParams;
pub use synchrotron_radiation::BetatronRadiation;
