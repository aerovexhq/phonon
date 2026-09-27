//! Electrochemical Metallization Cell (ECM) and Conductive Bridging RAM (CBRAM) model.
//!
//! Models:
//! 1. Butler-Volmer electric-field driven ionic hopping migration.
//! 2. Metallic filament nucleation, growth, and redox dissolution dynamics.
//! 3. Conductance quantization at single-atom constrictions (\(G = n \cdot G_0\)).
//! 4. Threshold switching (volatile) vs memory retention (non-volatile) regimes.

const Q_E: f64 = 1.602_176_634e-19; // Elementary charge [C]
const K_B: f64 = 1.380_649e-23; // Boltzmann constant [J/K]
const H_PLANCK: f64 = 6.626_070_15e-34; // Planck constant [J*s]
pub const G_0: f64 = 2.0 * Q_E * Q_E / H_PLANCK; // Landauer conductance quantum ~ 77.48 uS
pub const R_0: f64 = 1.0 / G_0; // Quantum resistance ~ 12.906 kOhm

/// Switching regime of the electrochemical metallization cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmSwitchingMode {
    /// Non-volatile state retention (filament remains stable at 0V)
    NonVolatileMemory,
    /// Volatile threshold switching (filament dissolves spontaneously at zero bias)
    VolatileThreshold,
}

/// Conduction state of the electrochemical filament.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EcmConductionState {
    /// Pristine high-resistance off state (tunneling across gap)
    HighResistanceOff,
    /// Active filament growth / dissolution in progress
    Transitioning,
    /// Single or few-atom quantized metallic bridge
    QuantizedContact { atomic_channels: usize },
    /// Fully established low-resistance metallic filament
    OhmicOn,
}

/// Physical parameters for an electrochemical metallization switch.
#[derive(Debug, Clone)]
pub struct EcmCellParameters {
    /// Solid electrolyte gap thickness L_gap [m] (e.g., 8.0 nm)
    pub gap_thickness_m: f64,
    /// Mean ion hopping distance a [m] (e.g., 0.35 nm)
    pub hopping_distance_m: f64,
    /// Phonon attempt frequency f_0 [1/s] (e.g., 1e12 s^-1)
    pub attempt_frequency_hz: f64,
    /// Ion migration activation energy E_a [eV]
    pub migration_barrier_ev: f64,
    /// Filament dissolution activation energy E_diss [eV]
    pub dissolution_barrier_ev: f64,
    /// Ion valence z (e.g., 1 for Ag+, 2 for Cu2+)
    pub ion_valence: f64,
    /// High-resistance off-state resistance [Ohm]
    pub r_off_ohm: f64,
    /// Series contact resistance [Ohm]
    pub r_series_ohm: f64,
    /// Metallic filament bulk resistivity [Ohm * m]
    pub filament_resistivity_ohm_m: f64,
    /// Critical filament radius for non-volatile stability [m]
    pub critical_radius_m: f64,
    /// Switching operational mode
    pub mode: EcmSwitchingMode,
}

impl Default for EcmCellParameters {
    fn default() -> Self {
        Self {
            gap_thickness_m: 8.0e-9,            // 8 nm solid electrolyte
            hopping_distance_m: 0.35e-9,        // 0.35 nm ionic hop
            attempt_frequency_hz: 1.0e12,       // 1 THz attempt frequency
            migration_barrier_ev: 0.38, // 0.38 eV superionic migration barrier (Ag2S / fast ECM)
            dissolution_barrier_ev: 0.48, // 0.48 eV dissolution barrier
            ion_valence: 1.0,           // Ag+ cations
            r_off_ohm: 1.0e11,          // 100 GOhm off-state
            r_series_ohm: 50.0,         // 50 Ohm electrode contact
            filament_resistivity_ohm_m: 2.0e-8, // Ag metallic resistivity
            critical_radius_m: 1.2e-9,  // 1.2 nm critical radius
            mode: EcmSwitchingMode::NonVolatileMemory,
        }
    }
}

/// Dynamic state of an electrochemical metallization cell.
#[derive(Debug, Clone)]
pub struct EcmCellState {
    /// Normalized filament length h in [0.0, 1.0] relative to gap thickness
    pub filament_length_ratio: f64,
    /// Filament radius r [m]
    pub filament_radius_m: f64,
    /// Number of conductive quantum atomic channels
    pub atomic_channels: usize,
    /// Operating junction temperature [K]
    pub temp_k: f64,
}

impl Default for EcmCellState {
    fn default() -> Self {
        Self {
            filament_length_ratio: 0.0,
            filament_radius_m: 0.0,
            atomic_channels: 0,
            temp_k: 300.0,
        }
    }
}

/// Physical model for electrochemical metallization and conductive bridging switching.
#[derive(Debug, Clone)]
pub struct EcmCellModel {
    pub params: EcmCellParameters,
    pub state: EcmCellState,
}

impl EcmCellModel {
    /// Creates a new ECM model with specified parameters.
    pub fn new(params: EcmCellParameters) -> Self {
        Self {
            params,
            state: EcmCellState::default(),
        }
    }

    /// Evaluates ionic hopping drift velocity [m/s] using Butler-Volmer kinetics.
    ///
    /// \[v_{ion}(E, T) = 2 a f_0 \exp\left(-\frac{E_a}{k_B T}\right) \sinh\left(\frac{z q a E}{2 k_B T}\right)\]
    pub fn evaluate_ion_drift_velocity(&self, v_bias: f64) -> f64 {
        let t_k = self.state.temp_k.max(100.0);
        let rem_gap_m =
            (self.params.gap_thickness_m * (1.0 - self.state.filament_length_ratio)).max(0.1e-9);
        let electric_field = v_bias / rem_gap_m; // [V/m]

        let e_a_j = self.params.migration_barrier_ev * Q_E;
        let thermal_energy = K_B * t_k;
        let arrhenius = (-e_a_j / thermal_energy).exp();

        let field_factor =
            (self.params.ion_valence * Q_E * self.params.hopping_distance_m * electric_field)
                / (2.0 * thermal_energy);

        2.0 * self.params.hopping_distance_m
            * self.params.attempt_frequency_hz
            * arrhenius
            * field_factor.sinh()
    }

    /// Evaluates filament dissolution velocity [m/s].
    pub fn evaluate_dissolution_velocity(&self, v_bias: f64) -> f64 {
        let t_k = self.state.temp_k.max(100.0);
        let thermal_energy = K_B * t_k;
        let e_diss_j = self.params.dissolution_barrier_ev * Q_E;

        // In non-volatile memory mode with stable filament, spontaneous dissolution is suppressed at zero/positive bias
        if self.params.mode == EcmSwitchingMode::NonVolatileMemory
            && self.state.filament_radius_m >= self.params.critical_radius_m
            && v_bias >= -0.02
        {
            return 0.0;
        }

        // Negative bias accelerates dissolution; positive bias suppresses it
        let bias_assist = if v_bias < 0.0 {
            (self.params.ion_valence * Q_E * v_bias.abs() * 0.5) / thermal_energy
        } else {
            0.0
        };

        // Spontaneous surface tension driving dissolution in volatile threshold mode or sub-critical filaments
        let surface_tension_assist = if self.state.filament_length_ratio >= 0.99
            && (self.params.mode == EcmSwitchingMode::VolatileThreshold
                || self.state.filament_radius_m < self.params.critical_radius_m)
        {
            0.15 * Q_E / thermal_energy
        } else {
            0.0
        };

        let exponent = (-(e_diss_j) / thermal_energy) + bias_assist + surface_tension_assist;
        self.params.hopping_distance_m * self.params.attempt_frequency_hz * exponent.min(50.0).exp()
    }

    /// Advances the filament growth/dissolution state across a time step dt [s].
    pub fn step(&mut self, v_bias: f64, dt_s: f64) {
        let v_growth = self.evaluate_ion_drift_velocity(v_bias);
        let v_diss = self.evaluate_dissolution_velocity(v_bias);

        let dh_dt = v_growth - v_diss;
        let dh = dh_dt * dt_s;

        let cur_h = self.state.filament_length_ratio * self.params.gap_thickness_m;
        let new_h = (cur_h + dh).clamp(0.0, self.params.gap_thickness_m);
        self.state.filament_length_ratio = new_h / self.params.gap_thickness_m;

        // When filament bridges the gap (h == L_gap), lateral growth/thinning occurs
        if self.state.filament_length_ratio >= 0.999 {
            if v_bias > 0.05 {
                // Lateral radial growth under forward current
                let dr = 0.5 * v_growth.abs() * dt_s * 0.1;
                self.state.filament_radius_m = (self.state.filament_radius_m + dr).min(5.0e-9);
                let cross_section = std::f64::consts::PI * self.state.filament_radius_m.powi(2);
                let atom_area = std::f64::consts::PI * (0.14e-9_f64).powi(2); // Ag atomic radius ~ 0.14 nm
                self.state.atomic_channels = ((cross_section / atom_area).round() as usize).max(1);
            } else if v_bias < -0.05 {
                // Lateral thinning under reverse bias
                let dr = 0.5 * v_diss * dt_s * 0.2;
                self.state.filament_radius_m = (self.state.filament_radius_m - dr).max(0.0);
                if self.state.filament_radius_m <= 0.14e-9 {
                    self.state.atomic_channels = 0;
                    self.state.filament_length_ratio = 0.98; // Break bridge
                } else {
                    let cross_section = std::f64::consts::PI * self.state.filament_radius_m.powi(2);
                    let atom_area = std::f64::consts::PI * (0.14e-9_f64).powi(2);
                    self.state.atomic_channels = (cross_section / atom_area).round() as usize;
                }
            }
        } else {
            self.state.atomic_channels = 0;
            self.state.filament_radius_m = 0.0;
        }
    }

    /// Current conduction state of the cell.
    pub fn conduction_state(&self) -> EcmConductionState {
        if self.state.filament_length_ratio < 0.1 {
            EcmConductionState::HighResistanceOff
        } else if self.state.filament_length_ratio < 0.999 {
            EcmConductionState::Transitioning
        } else if self.state.atomic_channels <= 5 {
            EcmConductionState::QuantizedContact {
                atomic_channels: self.state.atomic_channels,
            }
        } else {
            EcmConductionState::OhmicOn
        }
    }

    /// Evaluates equivalent electrical conductance G [Siemens].
    pub fn conductance_s(&self) -> f64 {
        match self.conduction_state() {
            EcmConductionState::HighResistanceOff => 1.0 / self.params.r_off_ohm,
            EcmConductionState::Transitioning => {
                // Direct tunneling resistance across residual vacuum gap
                let rem_ratio = 1.0 - self.state.filament_length_ratio;
                let r_tunnel = self.params.r_off_ohm * rem_ratio.powi(3).max(1e-6);
                1.0 / (r_tunnel + self.params.r_series_ohm)
            }
            EcmConductionState::QuantizedContact { atomic_channels } => {
                // Landauer conductance quantization G = n * G_0
                let g_quant = (atomic_channels.max(1) as f64) * G_0;
                let r_quant = 1.0 / g_quant;
                1.0 / (r_quant + self.params.r_series_ohm)
            }
            EcmConductionState::OhmicOn => {
                // Metallic cylinder resistance + Sharvin contact resistance
                let area = std::f64::consts::PI * self.state.filament_radius_m.powi(2);
                let r_bulk = self.params.filament_resistivity_ohm_m
                    * (self.params.gap_thickness_m / area.max(1e-20));
                1.0 / (r_bulk + self.params.r_series_ohm)
            }
        }
    }

    /// Evaluates equivalent electrical resistance R [Ohms].
    pub fn resistance_ohm(&self) -> f64 {
        1.0 / self.conductance_s().max(1e-18)
    }

    /// Current I [A] conducted at given terminal voltage V.
    pub fn current_a(&self, v_bias: f64) -> f64 {
        v_bias * self.conductance_s()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecm_conductance_quantization_at_bridge() {
        let mut cell = EcmCellModel::new(EcmCellParameters::default());
        assert_eq!(
            cell.conduction_state(),
            EcmConductionState::HighResistanceOff
        );
        assert!(cell.resistance_ohm() > 1.0e10);

        // Apply positive program bias to grow filament
        for _ in 0..100 {
            cell.step(0.60, 2.0e-6); // 600 mV program pulse
        }

        // Bridge formed
        assert!(cell.state.filament_length_ratio >= 0.99);
        assert!(cell.state.atomic_channels >= 1);

        // Resistance should be on the order of quantum / ohmic resistance
        let r = cell.resistance_ohm();
        assert!(r < 25_000.0, "Resistance was {} Ohm", r);
        assert!(r > 10.0);
    }

    #[test]
    fn test_ecm_non_volatile_retention_at_zero_bias() {
        let mut cell = EcmCellModel::new(EcmCellParameters::default());
        // Form bridge
        for _ in 0..100 {
            cell.step(0.60, 2.0e-6);
        }
        let r_initial = cell.resistance_ohm();
        assert!(
            r_initial < 1000.0,
            "Bridge should have formed, but r_initial was {}",
            r_initial
        );

        // Hold at 0V bias for 1000 steps
        for _ in 0..1000 {
            cell.step(0.0, 1.0e-6);
        }

        // Must retain low-resistance on-state
        let r_after_retention = cell.resistance_ohm();
        assert!((r_after_retention - r_initial).abs() < 50.0);
    }
}
