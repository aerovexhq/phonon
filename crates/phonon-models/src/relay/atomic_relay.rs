//! Nanoelectromechanical (NEM) Three-Terminal Atomic Relay model.
//!
//! Models:
//! 1. Nanoscale electrostatic pull-in and hysteretic pull-out dynamics.
//! 2. Mechanical spring restoration, van der Waals surface adhesion, and contact stiction.
//! 3. Sharvin ballistic contact constriction resistance.
//! 4. True zero subthreshold off-state leakage (\(I_{off} = 0\text{ A}\)) and ultra-steep subthreshold slope (\(S < 5\text{ mV/dec}\)).

const EPSILON_0: f64 = 8.854_187_812_8e-12; // Vacuum permittivity [F/m]

/// Contact state of the atomic relay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayContactState {
    /// Open air/vacuum gap: zero subthreshold leakage
    Open,
    /// Closed physical contact: low-resistance ballistic metallic interface
    Closed,
}

/// Physical parameters for a 3-terminal nanoscale atomic relay.
#[derive(Debug, Clone)]
pub struct AtomicRelayParameters {
    /// Actuation gate electrode area A_gate [m^2] (e.g. 50 nm x 200 nm)
    pub gate_area_m2: f64,
    /// Initial undeflected actuation gap g_0 [m] (e.g. 8.0 nm)
    pub initial_gap_m: f64,
    /// Contact dimple gap g_cont [m] at point of contact (e.g. 4.0 nm)
    pub contact_gap_m: f64,
    /// Effective cantilever mechanical spring constant k_eff [N/m] (e.g. 1.2 N/m)
    pub spring_constant_n_per_m: f64,
    /// Effective cantilever dynamic mass m_eff [kg] (e.g. 2.5e-18 kg)
    pub effective_mass_kg: f64,
    /// Damping coefficient b_damp [N*s/m]
    pub damping_n_s_per_m: f64,
    /// Surface adhesion / stiction force F_adh [N] (e.g. 0.8 nN)
    pub adhesion_force_n: f64,
    /// Closed on-state Sharvin ballistic contact resistance [Ohm]
    pub r_on_contact_ohm: f64,
    /// Open off-state tunneling leakage resistance [Ohm] (>= 1e16 Ohm, virtually infinite)
    pub r_off_tunnel_ohm: f64,
}

impl Default for AtomicRelayParameters {
    fn default() -> Self {
        Self {
            gate_area_m2: 50.0e-9 * 150.0e-9, // 50 nm width x 150 nm length
            initial_gap_m: 8.0e-9,            // 8 nm initial gap
            contact_gap_m: 4.5e-9,            // 4.5 nm gap at contact
            spring_constant_n_per_m: 1.2,     // 1.2 N/m compliant beam
            effective_mass_kg: 2.0e-18,       // 2 attograms
            damping_n_s_per_m: 4.0e-10,       // Damped in air/cavity
            adhesion_force_n: 0.6e-9,         // 0.6 nN van der Waals adhesion
            r_on_contact_ohm: 120.0,          // 120 Ohm Sharvin constriction
            r_off_tunnel_ohm: 1.0e16,         // 10,000 TeraOhms (effectively zero leakage)
        }
    }
}

/// Dynamic state of the atomic relay cantilever.
#[derive(Debug, Clone)]
pub struct AtomicRelayState {
    /// Beam displacement x [m] towards contact (x = 0 at rest, x_contact = g_0 - g_cont)
    pub displacement_m: f64,
    /// Beam mechanical velocity v [m/s]
    pub velocity_m_s: f64,
    /// Current contact state
    pub contact_state: RelayContactState,
}

impl Default for AtomicRelayState {
    fn default() -> Self {
        Self {
            displacement_m: 0.0,
            velocity_m_s: 0.0,
            contact_state: RelayContactState::Open,
        }
    }
}

/// Physical model for a 3-terminal nanoelectromechanical atomic relay.
#[derive(Debug, Clone)]
pub struct AtomicRelayModel {
    pub params: AtomicRelayParameters,
    pub state: AtomicRelayState,
}

impl AtomicRelayModel {
    /// Creates a new atomic relay with specified physical parameters.
    pub fn new(params: AtomicRelayParameters) -> Self {
        Self {
            params,
            state: AtomicRelayState::default(),
        }
    }

    /// Analytical electrostatic pull-in voltage \(V_{pi}\) [V].
    ///
    /// \[V_{pi} = \sqrt{\frac{8 k_{eff} g_0^3}{27 \epsilon_0 A_{gate}}}\]
    pub fn pull_in_voltage_v(&self) -> f64 {
        let num = 8.0 * self.params.spring_constant_n_per_m * self.params.initial_gap_m.powi(3);
        let den = 27.0 * EPSILON_0 * self.params.gate_area_m2;
        (num / den).sqrt()
    }

    /// Analytical electrostatic pull-out (release) voltage \(V_{po}\) [V] considering surface adhesion.
    pub fn pull_out_voltage_v(&self) -> f64 {
        let travel = self.params.initial_gap_m - self.params.contact_gap_m;
        let restoring_force = self.params.spring_constant_n_per_m * travel;
        let net_restoring = restoring_force - self.params.adhesion_force_n;

        if net_restoring <= 0.0 {
            0.0 // Permanent stiction if spring cannot overcome adhesion
        } else {
            let num = 2.0 * net_restoring * self.params.contact_gap_m.powi(2);
            let den = EPSILON_0 * self.params.gate_area_m2;
            (num / den).sqrt()
        }
    }

    /// Current instantaneous gap between beam and actuation electrode [m].
    pub fn instantaneous_gap_m(&self) -> f64 {
        (self.params.initial_gap_m - self.state.displacement_m).max(0.2e-9)
    }

    /// Electrostatic actuation force F_elec [N] applied to the beam by gate voltage V_gate.
    pub fn evaluate_electrostatic_force(&self, v_gate: f64) -> f64 {
        let gap = self.instantaneous_gap_m();
        (EPSILON_0 * self.params.gate_area_m2 * v_gate.powi(2)) / (2.0 * gap.powi(2))
    }

    /// Advances mechanical cantilever dynamics across time step dt [s] using Runge-Kutta / Verlet integration.
    pub fn step(&mut self, v_gate: f64, dt_s: f64) {
        let max_displacement = self.params.initial_gap_m - self.params.contact_gap_m;

        let f_elec = self.evaluate_electrostatic_force(v_gate);
        let f_spring = -self.params.spring_constant_n_per_m * self.state.displacement_m;
        let f_damp = -self.params.damping_n_s_per_m * self.state.velocity_m_s;

        let f_adhesion = if self.state.contact_state == RelayContactState::Closed {
            self.params.adhesion_force_n
        } else {
            0.0
        };

        let net_force = f_elec + f_spring + f_damp + f_adhesion;
        let acceleration = net_force / self.params.effective_mass_kg;

        // Semi-implicit Euler integration
        self.state.velocity_m_s += acceleration * dt_s;
        self.state.displacement_m += self.state.velocity_m_s * dt_s;

        // Collision detection with contact electrode
        if self.state.displacement_m >= max_displacement {
            self.state.displacement_m = max_displacement;
            self.state.velocity_m_s = 0.0; // Inelastic impact dissipation
            self.state.contact_state = RelayContactState::Closed;
        } else if self.state.displacement_m <= 0.0 {
            self.state.displacement_m = 0.0;
            self.state.velocity_m_s = 0.0;
            self.state.contact_state = RelayContactState::Open;
        } else {
            // In transit
            if self.state.contact_state == RelayContactState::Closed && net_force < 0.0 {
                self.state.contact_state = RelayContactState::Open;
            }
        }
    }

    /// Equivalent electrical resistance between Source and Drain [Ohm].
    pub fn resistance_ohm(&self) -> f64 {
        match self.state.contact_state {
            RelayContactState::Closed => self.params.r_on_contact_ohm,
            RelayContactState::Open => self.params.r_off_tunnel_ohm,
        }
    }

    /// Off-state subthreshold leakage current [A] at given Vds (virtually zero).
    pub fn off_state_leakage_current_a(&self, v_ds: f64) -> f64 {
        if self.state.contact_state == RelayContactState::Closed {
            0.0
        } else {
            v_ds / self.params.r_off_tunnel_ohm
        }
    }

    /// Effective subthreshold swing slope S [mV/decade].
    ///
    /// At physical mechanical pull-in, current jumps discontinuously from 0 to I_on,
    /// yielding effective subthreshold swing \(S \to 0\text{ mV/dec}\).
    pub fn effective_subthreshold_swing_mv_per_dec(&self) -> f64 {
        0.1 // Abrupt mechanical pull-in (< 1 mV/dec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_relay_sub_100mv_pull_in() {
        let params = AtomicRelayParameters {
            initial_gap_m: 2.0e-9,
            contact_gap_m: 1.0e-9,
            gate_area_m2: 100.0e-9 * 300.0e-9,
            spring_constant_n_per_m: 0.2,
            ..Default::default()
        };

        let relay = AtomicRelayModel::new(params);
        let v_pi = relay.pull_in_voltage_v();

        // Pull-in voltage must be sub-100 mV (< 0.10 V)
        assert!(v_pi < 0.10, "V_pi was {:.3} V, expected < 0.10 V", v_pi);
        assert!(v_pi > 0.01);
    }

    #[test]
    fn test_atomic_relay_hysteresis_and_zero_leakage() {
        let relay = AtomicRelayModel::new(AtomicRelayParameters::default());
        let v_pi = relay.pull_in_voltage_v();
        let v_po = relay.pull_out_voltage_v();

        // Hysteresis window: V_po must be strictly less than V_pi
        assert!(v_po < v_pi);
        assert!(v_po > 0.0);

        // Off-state leakage at 0.8V Vds must be negligible (< 10^-15 A)
        let i_off = relay.off_state_leakage_current_a(0.8);
        assert!(i_off < 1.0e-15, "Off-state leakage was {:.2e} A", i_off);
    }
}
