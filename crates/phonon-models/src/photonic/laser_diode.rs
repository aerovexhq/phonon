//! Semiconductor laser diode physical model and single-mode rate equations.
//!
//! Formulates:
//! - Coupled carrier-photon rate equations ($N(t)$, $S(t)$) with gain saturation and spontaneous emission.
//! - Static Light-Current (L-I) curve with characteristic temperature $T_0$ threshold shift:
//!   $$I_{th}(T) = I_{th,0} \exp\left( \frac{T - T_0}{T_0^{char}} \right)$$
//! - Dynamic relaxation oscillation frequency $f_r$ and damping factor $\gamma$.
//! - Electrical p-n diode junction companion model (conductance $g_d$, current $I_D$).

use phonon_core::{
    OpticalSignal, BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, PLANCK_CONSTANT, SPEED_OF_LIGHT, T_REF,
};

/// Physical parameters for a semiconductor laser diode (e.g. InP/InGaAsP DFB laser at 1550 nm).
#[derive(Debug, Clone, PartialEq)]
pub struct LaserDiodeModel {
    /// Active cavity length $L$ in meters ($m$).
    pub cavity_length_m: f64,
    /// Active region width $w$ in meters ($m$).
    pub active_width_m: f64,
    /// Active region thickness $d$ in meters ($m$).
    pub active_thickness_m: f64,
    /// Optical confinement factor $\Gamma$ ($0 < \Gamma \le 1$).
    pub confinement_factor: f64,
    /// Differential optical gain coefficient $a_0 = dg/dN$ in $\text{m}^2$.
    pub differential_gain_m2: f64,
    /// Carrier density at transparency $N_0$ in $\text{m}^{-3}$.
    pub transparency_density_m3: f64,
    /// Carrier recombination lifetime $\tau_n$ in seconds ($s$).
    pub carrier_lifetime_s: f64,
    /// Photon cavity lifetime $\tau_p$ in seconds ($s$).
    pub photon_lifetime_s: f64,
    /// Gain compression factor $\epsilon$ in $\text{m}^3$.
    pub gain_compression_m3: f64,
    /// Spontaneous emission coupling factor $\beta_{sp}$.
    pub spontaneous_factor: f64,
    /// Internal quantum efficiency $\eta_i$.
    pub internal_quantum_efficiency: f64,
    /// Nominal threshold current $I_{th,0}$ at $T_0 = 300\text{ K}$ in Amperes ($A$).
    pub threshold_current_0_a: f64,
    /// Characteristic temperature $T_0^{char}$ in Kelvin ($K$) (governs $I_{th}(T)$ exponential shift).
    pub t0_characteristic_k: f64,
    /// Differential slope efficiency $\eta_{slope}$ at $300\text{ K}$ in Watts per Ampere ($W/A$).
    pub slope_efficiency_w_per_a: f64,
    /// Peak emission wavelength $\lambda_0$ in meters ($m$) ($1.55\,\mu\text{m}$).
    pub emission_wavelength_m: f64,
    /// Group refractive index $n_g$ of laser cavity.
    pub group_index_ng: f64,
    /// Reverse saturation current $I_s$ in Amperes ($A$).
    pub is_sat_a: f64,
    /// Junction ideality factor $n_{diode}$.
    pub ideality_factor: f64,
    /// Series resistance $R_s$ in Ohms ($\Omega$).
    pub series_resistance_ohms: f64,
}

/// Dynamic transient state of the laser cavity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaserDiodeState {
    /// Carrier density $N$ in active cavity ($\text{m}^{-3}$).
    pub carrier_density_m3: f64,
    /// Photon density $S$ in active cavity ($\text{m}^{-3}$).
    pub photon_density_m3: f64,
}

impl LaserDiodeModel {
    /// Standard InP/InGaAsP Distributed Feedback (DFB) laser diode preset at $1550\text{ nm}$.
    pub fn dfb_1550nm() -> Self {
        Self {
            cavity_length_m: 300e-6,
            active_width_m: 2.0e-6,
            active_thickness_m: 0.15e-6,
            confinement_factor: 0.35,
            differential_gain_m2: 2.5e-20,
            transparency_density_m3: 1.0e24,
            carrier_lifetime_s: 1.0e-9, // 1 ns
            photon_lifetime_s: 2.0e-12, // 2 ps
            gain_compression_m3: 3.0e-23,
            spontaneous_factor: 1.0e-4,
            internal_quantum_efficiency: 0.85,
            threshold_current_0_a: 15.0e-3, // 15 mA
            t0_characteristic_k: 60.0,      // 60 K typical for 1550 nm InP
            slope_efficiency_w_per_a: 0.25, // 0.25 W/A
            emission_wavelength_m: 1.55e-6,
            group_index_ng: 3.7,
            is_sat_a: 1e-12,
            ideality_factor: 1.5,
            series_resistance_ohms: 3.5,
        }
    }

    /// Active volume $V_{act} = L \cdot w \cdot d$ in $\text{m}^3$.
    #[inline]
    pub fn active_volume_m3(&self) -> f64 {
        self.cavity_length_m * self.active_width_m * self.active_thickness_m
    }

    /// Optical group velocity $v_g = c / n_g$ in meters per second ($m/s$).
    #[inline]
    pub fn group_velocity(&self) -> f64 {
        SPEED_OF_LIGHT / self.group_index_ng
    }

    /// Temperature-dependent threshold current:
    /// $$I_{th}(T) = I_{th,0} \exp\left( \frac{T - T_0}{T_0^{char}} \right)$$
    pub fn threshold_current(&self, temp_k: f64) -> f64 {
        let delta_t = temp_k - T_REF;
        self.threshold_current_0_a * (delta_t / self.t0_characteristic_k.max(1.0)).exp()
    }

    /// Temperature-dependent slope efficiency:
    /// Drops with temperature due to non-radiative Auger recombination and carrier leakage:
    pub fn slope_efficiency(&self, temp_k: f64) -> f64 {
        let delta_t = temp_k - T_REF;
        let factor = (-delta_t / (2.0 * self.t0_characteristic_k.max(1.0))).exp();
        self.slope_efficiency_w_per_a * factor
    }

    /// Static optical output power $P_{opt}(I, T)$ from injection current $I$ and temperature $T$:
    pub fn static_optical_power(&self, current_a: f64, temp_k: f64) -> f64 {
        let i_th = self.threshold_current(temp_k);
        let slope = self.slope_efficiency(temp_k);

        if current_a > i_th {
            let p_laser = slope * (current_a - i_th);
            let p_spont = slope * i_th * self.spontaneous_factor * (current_a / i_th);
            p_laser + p_spont
        } else {
            // Below threshold: purely spontaneous LED emission proportional to I
            let p_spont_max = slope * i_th * self.spontaneous_factor;
            p_spont_max * (current_a / i_th.max(1e-6)).max(0.0)
        }
    }

    /// Small-signal relaxation oscillation resonance frequency $f_r$ in Hertz ($Hz$):
    /// $$f_r = \frac{1}{2\pi} \sqrt{\frac{\Gamma v_g a_0 S}{\tau_p}} \propto \sqrt{I - I_{th}}$$
    pub fn relaxation_oscillation_frequency(&self, current_a: f64, temp_k: f64) -> f64 {
        let i_th = self.threshold_current(temp_k);
        if current_a <= i_th {
            return 0.0;
        }

        let delta_i = current_a - i_th;
        let v_act = self.active_volume_m3();
        let s_density = (self.internal_quantum_efficiency * delta_i * self.photon_lifetime_s)
            / (ELEMENTARY_CHARGE * v_act);

        let vg = self.group_velocity();
        let inside_sqrt = (self.confinement_factor * vg * self.differential_gain_m2 * s_density)
            / self.photon_lifetime_s;

        (inside_sqrt.max(0.0).sqrt()) / (2.0 * std::f64::consts::PI)
    }

    /// Evaluates rate equations derivative `[dN/dt, dS/dt]`:
    /// $$\frac{dN}{dt} = \frac{\eta_i I}{q V_{act}} - \frac{N}{\tau_n} - v_g \frac{a_0 (N - N_0)}{1 + \epsilon S} S$$
    /// $$\frac{dS}{dt} = \Gamma v_g \frac{a_0 (N - N_0)}{1 + \epsilon S} S - \frac{S}{\tau_p} + \Gamma \beta_{sp} \frac{N}{\tau_n}$$
    pub fn rate_derivatives(&self, state: LaserDiodeState, current_a: f64) -> (f64, f64) {
        let v_act = self.active_volume_m3();
        let vg = self.group_velocity();
        let n = state.carrier_density_m3.max(0.0);
        let s = state.photon_density_m3.max(0.0);

        let injection_rate =
            (self.internal_quantum_efficiency * current_a) / (ELEMENTARY_CHARGE * v_act);
        let carrier_recomb = n / self.carrier_lifetime_s;

        // Optical gain: g(N, S) = a0 * (N - N0) / (1 + eps * S)
        let gain = (self.differential_gain_m2 * (n - self.transparency_density_m3))
            / (1.0 + self.gain_compression_m3 * s);

        let stimulated_emission = vg * gain * s;
        let spontaneous_into_mode =
            self.confinement_factor * self.spontaneous_factor * (n / self.carrier_lifetime_s);
        let photon_loss = s / self.photon_lifetime_s;

        let dn_dt = injection_rate - carrier_recomb - stimulated_emission;
        let ds_dt =
            self.confinement_factor * stimulated_emission - photon_loss + spontaneous_into_mode;

        (dn_dt, ds_dt)
    }

    /// Integrates rate equations forward by time-step $dt$ using Runge-Kutta 4th order (RK4).
    pub fn step_rk4(&self, state: LaserDiodeState, current_a: f64, dt_s: f64) -> LaserDiodeState {
        let (k1_n, k1_s) = self.rate_derivatives(state, current_a);

        let state_k2 = LaserDiodeState {
            carrier_density_m3: state.carrier_density_m3 + 0.5 * dt_s * k1_n,
            photon_density_m3: state.photon_density_m3 + 0.5 * dt_s * k1_s,
        };
        let (k2_n, k2_s) = self.rate_derivatives(state_k2, current_a);

        let state_k3 = LaserDiodeState {
            carrier_density_m3: state.carrier_density_m3 + 0.5 * dt_s * k2_n,
            photon_density_m3: state.photon_density_m3 + 0.5 * dt_s * k2_s,
        };
        let (k3_n, k3_s) = self.rate_derivatives(state_k3, current_a);

        let state_k4 = LaserDiodeState {
            carrier_density_m3: state.carrier_density_m3 + dt_s * k3_n,
            photon_density_m3: state.photon_density_m3 + dt_s * k3_s,
        };
        let (k4_n, k4_s) = self.rate_derivatives(state_k4, current_a);

        let next_n =
            state.carrier_density_m3 + (dt_s / 6.0) * (k1_n + 2.0 * k2_n + 2.0 * k3_n + k4_n);
        let next_s =
            state.photon_density_m3 + (dt_s / 6.0) * (k1_s + 2.0 * k2_s + 2.0 * k3_s + k4_s);

        LaserDiodeState {
            carrier_density_m3: next_n.max(0.0),
            photon_density_m3: next_s.max(0.0),
        }
    }

    /// Converts internal cavity photon density $S$ into output optical power in Watts ($W$):
    /// $$P_{opt} = \frac{1}{2} v_g \alpha_m h \nu V_{act} S$$
    pub fn photon_density_to_power(&self, photon_density_m3: f64) -> f64 {
        let photon_energy = (PLANCK_CONSTANT * SPEED_OF_LIGHT) / self.emission_wavelength_m;
        // Extraction rate per photon = 1 / (2 * tau_p)
        let extraction_rate = 1.0 / (2.0 * self.photon_lifetime_s);
        photon_density_m3 * self.active_volume_m3() * extraction_rate * photon_energy
    }

    /// Evaluates electrical forward I-V characteristic and dynamic conductance $g_d = dI/dV$:
    /// Used for Modified Nodal Analysis (MNA) Newton-Raphson Jacobian stamping.
    pub fn electrical_companion(&self, v_diode_volts: f64, temp_k: f64) -> (f64, f64) {
        let v_t = (BOLTZMANN_CONSTANT * temp_k) / ELEMENTARY_CHARGE;
        let n_vt = self.ideality_factor * v_t;

        // Limit exponent for numerical safety
        let v_clamped = v_diode_volts.clamp(-10.0, 3.0);
        let exp_term = (v_clamped / n_vt).min(40.0).exp();

        let i_diode = self.is_sat_a * (exp_term - 1.0);
        let g_diode = (self.is_sat_a * exp_term) / n_vt;

        (i_diode, g_diode)
    }

    /// Generates an output OpticalSignal from driving current $I$ and temperature $T$.
    pub fn emit_signal(&self, current_a: f64, temp_k: f64) -> OpticalSignal {
        let p_opt = self.static_optical_power(current_a, temp_k);
        OpticalSignal {
            wavelength_m: self.emission_wavelength_m,
            power_watts: p_opt,
            phase_rad: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfb_laser_threshold_and_li_curve() {
        let laser = LaserDiodeModel::dfb_1550nm();

        // At T_REF, threshold is 15 mA
        let i_th = laser.threshold_current(T_REF);
        assert!((i_th - 0.015).abs() < 1e-6);

        // At 10 mA (below threshold), power is minimal spontaneous emission
        let p_sub = laser.static_optical_power(0.010, T_REF);
        assert!(p_sub < 1e-4);

        // At 35 mA (20 mA above threshold), power ~ slope * 20 mA = 0.25 * 0.02 = 5 mW
        let p_above = laser.static_optical_power(0.035, T_REF);
        assert!((p_above - 5e-3).abs() < 1e-4);

        // Higher temperature (T_REF + 60 K) shifts threshold up: Ith = 15 mA * exp(60/60) = 15 mA * e ~ 40.77 mA
        let i_th_hot = laser.threshold_current(T_REF + 60.0);
        assert!((i_th_hot - 0.015 * std::f64::consts::E).abs() < 1e-4);
    }

    #[test]
    fn test_relaxation_oscillations_scaling() {
        let laser = LaserDiodeModel::dfb_1550nm();
        // fr at 35 mA (20 mA above threshold)
        let fr_35ma = laser.relaxation_oscillation_frequency(0.035, 300.0);
        // fr at 95 mA (80 mA above threshold, 4x current -> 2x fr)
        let fr_95ma = laser.relaxation_oscillation_frequency(0.095, 300.0);

        assert!(fr_35ma > 1.0e9); // Multi-GHz relaxation oscillations
        let ratio = fr_95ma / fr_35ma;
        assert!((ratio - 2.0).abs() < 0.15); // sqrt(4) = 2 scaling
    }

    #[test]
    fn test_dynamic_rate_equations_rk4() {
        let laser = LaserDiodeModel::dfb_1550nm();
        let mut state = LaserDiodeState {
            carrier_density_m3: 0.0,
            photon_density_m3: 0.0,
        };

        // Step transient for 100 ps at 40 mA
        let dt = 1e-12; // 1 ps
        for _ in 0..100 {
            state = laser.step_rk4(state, 0.040, dt);
        }

        // Carriers should have built up towards threshold
        assert!(state.carrier_density_m3 > 1e23);
    }
}
