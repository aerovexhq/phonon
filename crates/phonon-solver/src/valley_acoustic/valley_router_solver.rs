//! Wavepacket scattering solver for valley acoustic waveguide multiplexers,
//! sharp corner propagation, and chiral phonon routers.

use phonon_models::valley_acoustic::{ValleyAcousticRouter, ValleyIndex, ValleyRoutingResult};

/// Wavepacket routing and scattering solver for valley acoustic networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyRouterSolver {
    pub router: ValleyAcousticRouter,
}

impl ValleyRouterSolver {
    /// Creates a new valley router solver.
    pub fn new(router: ValleyAcousticRouter) -> Self {
        Self { router }
    }

    /// Solves the 3-port routing distribution for an incident wavepacket.
    pub fn solve_routing(&self, input_valley: ValleyIndex) -> ValleyRoutingResult {
        self.router.route_wavepacket(input_valley)
    }

    /// Solves transmission around a sharp waveguide corner bend ($T_{\mathrm{bend}} \ge 90\%$).
    pub fn solve_corner_transmission(&self) -> f64 {
        let res = self.router.route_wavepacket(ValleyIndex::ValleyK);
        res.corner_bend_transmission
    }
}
