//! Heavy-ion particle strike, Linear Energy Transfer (LET),
//! electron-hole pair track generation, and Single-Event Transient (SET) current pulse.
//!
//! Formulates:
//! - Physical electron-hole generation in silicon:
//!   $$\frac{dQ}{dx} = \frac{q \cdot \rho_{Si} \cdot \text{LET}}{E_{eh}} \approx 10.36 \times \text{LET} \quad [\text{fC}/\mu\text{m}]$$
//! - Funneling effect extending effective charge collection depth:
//!   $$L_{eff} = W_{dep} + L_{funnel}$$
//! - Total collected charge $Q_{coll} = \eta_{coll} \cdot \frac{dQ}{dx} \cdot L_{eff}$.
//! - Double-exponential current pulse (Messenger / Messenger-Baze formulation):
//!   $$I_{SET}(t) = \frac{Q_{coll}}{\tau_{fall} - \tau_{rise}} \left(\exp\left(-\frac{t - t_0}{\tau_{fall}}\right) - \exp\left(-\frac{t - t_0}{\tau_{rise}}\right)\right) \cdot \Theta(t - t_0)$$
//! - Analytical integral identity: $\int_{t_0}^\infty I_{SET}(t) dt \equiv Q_{coll}$.
//! - Critical charge threshold $Q_{crit}$ for Single-Event Upset (SEU) state bit-flips.

use phonon_core::{ELEMENTARY_CHARGE, SILICON_DENSITY};

/// Average energy required to generate one electron-hole pair in silicon at 300 K in Joules ($3.6\text{ eV}$).
pub const E_EH_SILICON_JOULES: f64 = 3.6 * ELEMENTARY_CHARGE;

/// Physical parameters for a cosmic heavy-ion strike and charge collection at a sensitive reverse-biased junction.
#[derive(Debug, Clone, PartialEq)]
pub struct HeavyIonStrikeModel {
    /// Linear Energy Transfer (LET) in $\text{MeV}\cdot\text{cm}^2/\text{mg}$ (typically $1 - 100\text{ MeV}\cdot\text{cm}^2/\text{mg}$).
    pub let_mev_cm2_mg: f64,
    /// Strike event timestamp $t_0$ in seconds ($s$).
    pub strike_time_s: f64,
    /// Depletion region physical width $W_{dep}$ in meters ($m$) (typically $0.1 - 0.5\,\mu\text{m}$).
    pub depletion_width_m: f64,
    /// Electric field funneling length $L_{funnel}$ in meters ($m$) (typically $0.5 - 2.0\,\mu\text{m}$).
    pub funneling_length_m: f64,
    /// Current pulse rise time constant $\tau_{rise}$ in seconds ($s$) (typically $5 - 20\text{ ps}$).
    pub tau_rise_s: f64,
    /// Current pulse fall time constant $\tau_{fall}$ in seconds ($s$) (typically $50 - 300\text{ ps}$).
    pub tau_fall_s: f64,
    /// Collection efficiency $\eta_{coll} \in [0, 1]$ (typically $0.85 - 1.0$).
    pub collection_efficiency: f64,
}

impl HeavyIonStrikeModel {
    /// Creates a typical deep-submicron cosmic heavy-ion strike preset (LET = $30\text{ MeV}\cdot\text{cm}^2/\text{mg}$).
    pub fn typical_30nm_heavy_ion(strike_time_s: f64, let_mev: f64) -> Self {
        Self {
            let_mev_cm2_mg: let_mev.max(0.1),
            strike_time_s,
            depletion_width_m: 0.20e-6,  // 200 nm
            funneling_length_m: 0.80e-6, // 800 nm funneling depth
            tau_rise_s: 10.0e-12,        // 10 ps rise
            tau_fall_s: 100.0e-12,       // 100 ps fall
            collection_efficiency: 0.95,
        }
    }

    /// Linear charge generation density $\frac{dQ}{dx}$ in Coulombs per meter ($\text{C/m}$).
    ///
    /// $$1\text{ MeV}\cdot\text{cm}^2/\text{mg} = 10^6 \times 1.602 \times 10^{-19}\text{ J}\cdot\text{cm}^2 / 10^{-3}\text{ kg}$$
    #[inline]
    pub fn linear_charge_density_c_per_m(&self) -> f64 {
        // Density of silicon: 2.328 g/cm^3 = 2328 kg/m^3
        // 1 MeV/(mg/cm^2) in SI: (1e6 * e) / (1e-6 kg / 1e-4 m^2) = 1e6 * e * 100 J*m^2/kg
        // Energy loss per unit path length: dE/dx = LET * rho_Si
        // dQ/dx = (q / E_eh) * dE/dx
        let let_si = self.let_mev_cm2_mg * 1.602_176_634e-13 * 100.0; // J * m^2 / kg
        let de_dx = let_si * SILICON_DENSITY; // J / m
        let pairs_per_m = de_dx / E_EH_SILICON_JOULES;
        ELEMENTARY_CHARGE * pairs_per_m
    }

    /// Effective collection depth $L_{eff} = W_{dep} + L_{funnel}$ in meters ($m$).
    #[inline(always)]
    pub fn effective_collection_depth_m(&self) -> f64 {
        self.depletion_width_m + self.funneling_length_m
    }

    /// Total collected charge $Q_{coll}$ in Coulombs ($C$).
    #[inline]
    pub fn total_collected_charge_c(&self) -> f64 {
        let dq_dx = self.linear_charge_density_c_per_m();
        let l_eff = self.effective_collection_depth_m();
        self.collection_efficiency * dq_dx * l_eff
    }

    /// Instantaneous Single-Event Transient (SET) current $I_{SET}(t)$ in Amperes ($A$).
    /// Current flows out of the affected junction node into the bulk/ground.
    pub fn current_at_time(&self, time_s: f64) -> f64 {
        if time_s <= self.strike_time_s {
            return 0.0;
        }

        let dt = time_s - self.strike_time_s;
        let q_tot = self.total_collected_charge_c();
        let tau_diff = (self.tau_fall_s - self.tau_rise_s).max(1e-15);

        let i_peak_factor = q_tot / tau_diff;
        let exp_fall = (-dt / self.tau_fall_s).exp();
        let exp_rise = (-dt / self.tau_rise_s).exp();

        (i_peak_factor * (exp_fall - exp_rise)).max(0.0)
    }

    /// Peak current of the SET pulse in Amperes ($A$).
    pub fn peak_current_a(&self) -> f64 {
        let tau_r = self.tau_rise_s;
        let tau_f = self.tau_fall_s;
        let t_peak = (tau_r * tau_f / (tau_f - tau_r)) * (tau_f / tau_r).ln();
        self.current_at_time(self.strike_time_s + t_peak)
    }

    /// Checks whether the deposited charge exceeds a target circuit node's critical charge $Q_{crit}$.
    #[inline]
    pub fn causes_single_event_upset(&self, q_crit_c: f64) -> bool {
        self.total_collected_charge_c() >= q_crit_c
    }
}

/// Evaluates node critical charge $Q_{crit} = C_{node} V_{dd} + I_{restore} \tau_{delay}$ for static memory cells.
#[inline]
pub fn evaluate_critical_charge(
    c_node_f: f64,
    v_dd_volts: f64,
    i_restore_a: f64,
    tau_delay_s: f64,
) -> f64 {
    c_node_f * v_dd_volts + i_restore_a * tau_delay_s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silicon_charge_generation_scaling() {
        let strike = HeavyIonStrikeModel::typical_30nm_heavy_ion(1.0e-9, 10.0); // LET = 10 MeV*cm^2/mg
        let dq_dx_fc_um = strike.linear_charge_density_c_per_m() * 1e15 * 1e-6; // fC/um

        // Standard physics: 1 MeV*cm^2/mg in Si yields ~ 10.36 fC/um
        // For LET = 10, expected ~ 103.6 fC/um
        assert!(
            (dq_dx_fc_um - 103.6).abs() < 1.0,
            "Linear charge density mismatch: expected ~103.6 fC/um, got {}",
            dq_dx_fc_um
        );
    }

    #[test]
    fn test_current_pulse_charge_integral_conservation() {
        let strike = HeavyIonStrikeModel::typical_30nm_heavy_ion(0.0, 20.0);
        let q_expected = strike.total_collected_charge_c();

        // Numerically integrate I(t) dt from 0 to 1 ns (10 * tau_fall) using trapezoidal rule
        let dt = 1e-13; // 100 fs
        let n_steps = 10000;
        let mut q_integrated = 0.0;

        for step in 0..n_steps {
            let t1 = (step as f64) * dt;
            let t2 = t1 + dt;
            let i1 = strike.current_at_time(t1);
            let i2 = strike.current_at_time(t2);
            q_integrated += 0.5 * (i1 + i2) * dt;
        }

        let rel_err = (q_integrated - q_expected).abs() / q_expected;
        assert!(
            rel_err < 1e-3,
            "Charge conservation violated: integrated = {} C, expected = {} C, rel_err = {}",
            q_integrated,
            q_expected,
            rel_err
        );
    }

    #[test]
    fn test_seu_critical_charge_threshold() {
        let strike = HeavyIonStrikeModel::typical_30nm_heavy_ion(0.0, 50.0); // Heavy strike
        let q_crit_sram = 50.0e-15; // 50 fC critical charge
        assert!(strike.causes_single_event_upset(q_crit_sram));

        let small_strike = HeavyIonStrikeModel::typical_30nm_heavy_ion(0.0, 1.0); // Proton recoil
        assert!(!small_strike.causes_single_event_upset(q_crit_sram));
    }
}
