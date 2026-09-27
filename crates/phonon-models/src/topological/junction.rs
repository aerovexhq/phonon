//! Multi-arm T-junction and Y-junction nanowire networks for Majorana exchange.
//!
//! Models interconnected nanowire geometries with electrostatic depletion gates
//! that translate topological/trivial boundaries and steer Majorana zero modes
//! without closing the bulk superconducting minigap $\Delta_{top}$.

use super::nanowire::NanowireParams;

/// Arm identifier in a T-junction or Y-junction network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmId {
    Left,
    Right,
    Stem,
}

/// Nanowire T-junction network configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct TJunctionNanowireNetwork {
    /// Physical length of each arm in nanometers.
    pub arm_length_nm: f64,
    /// Number of discretized sites per arm.
    pub sites_per_arm: usize,
    /// Underlying material and superconducting parameters.
    pub base_params: NanowireParams,
    /// Chemical potential $\mu_{Left}$ (meV) controlled by left electrostatic gate.
    pub mu_left_mev: f64,
    /// Chemical potential $\mu_{Right}$ (meV) controlled by right electrostatic gate.
    pub mu_right_mev: f64,
    /// Chemical potential $\mu_{Stem}$ (meV) controlled by stem electrostatic gate.
    pub mu_stem_mev: f64,
    /// Normalized positions of active Majorana Zero Modes along their respective arms:
    /// `(ArmId, position_in_0_to_1)` where 0.0 is the outer tip and 1.0 is the junction intersection.
    pub mzm1_location: (ArmId, f64),
    pub mzm2_location: (ArmId, f64),
}

impl Default for TJunctionNanowireNetwork {
    fn default() -> Self {
        let base = NanowireParams::default();
        Self {
            arm_length_nm: 1000.0, // 1 um arms
            sites_per_arm: 25,
            base_params: base,
            mu_left_mev: 0.0,  // Topological: V_Z = 1.2 > sqrt(0.5^2 + 0^2) = 0.5
            mu_right_mev: 0.0, // Topological
            mu_stem_mev: 3.0,  // Trivial/Depleted: V_Z = 1.2 < sqrt(0.5^2 + 3^2) = 3.04
            mzm1_location: (ArmId::Left, 0.0), // At far tip of Left arm
            mzm2_location: (ArmId::Right, 0.0), // At far tip of Right arm
        }
    }
}

impl TJunctionNanowireNetwork {
    /// Checks if a given arm is in the topological phase under its current gate voltage.
    pub fn is_arm_topological(&self, arm: ArmId) -> bool {
        let mu = match arm {
            ArmId::Left => self.mu_left_mev,
            ArmId::Right => self.mu_right_mev,
            ArmId::Stem => self.mu_stem_mev,
        };
        let vz = self.base_params.zeeman_mev;
        let delta = self.base_params.pairing_delta_mev;
        vz > (delta * delta + mu * mu).sqrt()
    }

    /// Evaluates the protective topological minigap of the active topological segments in meV.
    pub fn network_minigap_mev(&self) -> f64 {
        let delta = self.base_params.pairing_delta_mev;
        let vz = self.base_params.zeeman_mev;

        // Gap of the active topological segments hosting the Majoranas (mu ~ 0)
        let v_crit = delta;
        (vz - v_crit).abs().min(delta)
    }

    /// Updates gate voltages across all three arms.
    pub fn set_gates(&mut self, mu_left: f64, mu_right: f64, mu_stem: f64) {
        self.mu_left_mev = mu_left;
        self.mu_right_mev = mu_right;
        self.mu_stem_mev = mu_stem;
    }

    /// Sets the spatial locations of the two Majorana zero modes.
    pub fn set_mzm_locations(&mut self, loc1: (ArmId, f64), loc2: (ArmId, f64)) {
        self.mzm1_location = (loc1.0, loc1.1.clamp(0.0, 1.0));
        self.mzm2_location = (loc2.0, loc2.1.clamp(0.0, 1.0));
    }

    /// Euclidean distance between MZM1 and MZM2 in nanometers.
    /// Used to verify that Majoranas remain sufficiently separated to prevent hybridization.
    pub fn mzm_separation_nm(&self) -> f64 {
        let l = self.arm_length_nm;
        let (arm1, pos1) = self.mzm1_location;
        let (arm2, pos2) = self.mzm2_location;

        if arm1 == arm2 {
            (pos1 - pos2).abs() * l
        } else {
            // Distance from junction intersection for each mode: (1.0 - pos) * L
            let d1 = (1.0 - pos1) * l;
            let d2 = (1.0 - pos2) * l;
            match (arm1, arm2) {
                (ArmId::Left, ArmId::Right) | (ArmId::Right, ArmId::Left) => d1 + d2,
                _ => (d1 * d1 + d2 * d2).sqrt(),
            }
        }
    }
}
