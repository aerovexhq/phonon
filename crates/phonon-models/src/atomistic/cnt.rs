//! Carbon Nanotube (CNT) atomistic physics model.
//!
//! Formulates:
//! - Chiral vector $\mathbf{C}_h = n \mathbf{a}_1 + m \mathbf{a}_2$, tube diameter $d_t$, chiral angle $\theta$.
//! - Electronic character classification:
//!   - Armchair $(n, n)$: strictly metallic ($E_g = 0$).
//!   - Zigzag $(3q, 0)$ / Metallic chiral $(n - m \equiv 0 \pmod 3)$: quasi-metallic with curvature bandgap $\Delta E_{curv}$.
//!   - Semiconducting $(n - m \not\equiv 0 \pmod 3)$: bandgap $E_g \approx \frac{2 a_{cc} \gamma_0}{d_t}$.
//! - Zone-folded 1D subbands from graphene tight-binding dispersion.
//! - Quasi-ballistic Multi-subband Landauer transport with optical phonon emission velocity saturation.

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, PLANCK_CONSTANT};

/// Carbon-carbon bond length in graphene / CNT ($a_{cc} = 0.142\text{ nm}$).
pub const CARBON_BOND_LENGTH_M: f64 = 1.42e-10;

/// Nearest-neighbor carbon-carbon tight-binding hopping energy ($\gamma_0 \approx 2.7\text{ eV}$).
pub const GRAPHENE_HOPPING_EV: f64 = 2.7;

/// Quantum of conductance $G_0 = \frac{2 e^2}{h} \approx 7.74809 \times 10^{-5}\text{ S}$.
pub const QUANTUM_CONDUCTANCE_SI: f64 =
    2.0 * ELEMENTARY_CHARGE * ELEMENTARY_CHARGE / PLANCK_CONSTANT;

/// Theoretical 2-channel ballistic quantum resistance $R_Q = \frac{h}{4 e^2} \approx 6453.2\ \Omega$.
pub const QUANTUM_RESISTANCE_CNT_OHMS: f64 = 1.0 / (2.0 * QUANTUM_CONDUCTANCE_SI);

/// Electronic character of a Carbon Nanotube.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CntCharacter {
    /// Strictly metallic (e.g. armchair $(n, n)$ tubes).
    Metallic,
    /// Quasi-metallic with small curvature-induced secondary bandgap $\Delta E_{curv} < 100\text{ meV}$.
    QuasiMetallic,
    /// Semiconducting with primary bandgap $E_g \approx \frac{0.84\text{ eV}}{d_t\text{ [nm]}}$.
    Semiconducting,
}

/// Physical specification of a Single-Walled Carbon Nanotube (SWCNT).
#[derive(Debug, Clone, PartialEq)]
pub struct CarbonNanotube {
    /// Chiral index $n$.
    pub n: u32,
    /// Chiral index $m$.
    pub m: u32,
    /// Nanotube diameter $d_t$ in meters ($m$).
    pub diameter_m: f64,
    /// Chiral angle $\theta$ in radians ($0 \le \theta \le \pi/6$).
    pub chiral_angle_rad: f64,
    /// Electronic classification (Metallic, QuasiMetallic, Semiconducting).
    pub character: CntCharacter,
    /// First subband bandgap $E_g$ in electron-volts ($eV$).
    pub bandgap_ev: f64,
    /// Tight-binding hopping parameter $\gamma_0$ in $eV$.
    pub hopping_gamma0_ev: f64,
    /// Acoustic phonon mean free path at 300K in meters ($\approx 1\ \mu\text{m}$).
    pub lambda_acoustic_300k_m: f64,
    /// Optical phonon energy in $eV$ ($\hbar \omega_{OP} \approx 0.16\text{ eV}$).
    pub hbar_omega_op_ev: f64,
}

impl CarbonNanotube {
    /// Constructs a Carbon Nanotube from chirality indices $(n, m)$ with default graphene parameters.
    pub fn new(n: u32, m: u32) -> Self {
        Self::with_hopping(n, m, GRAPHENE_HOPPING_EV)
    }

    /// Constructs an Armchair $(n, n)$ metallic nanotube.
    pub fn armchair(n: u32) -> Self {
        Self::new(n, n)
    }

    /// Constructs a Zigzag $(n, 0)$ nanotube.
    pub fn zigzag(n: u32) -> Self {
        Self::new(n, 0)
    }

    /// Constructs a CNT with custom tight-binding hopping energy $\gamma_0$.
    pub fn with_hopping(n: u32, m: u32, gamma0_ev: f64) -> Self {
        let (n_val, m_val) = if n >= m { (n, m) } else { (m, n) };

        // Tube diameter: d_t = (a_cc * sqrt(3) / pi) * sqrt(n^2 + nm + m^2)
        let a = CARBON_BOND_LENGTH_M * 3.0_f64.sqrt();
        let c_norm = ((n_val * n_val + n_val * m_val + m_val * m_val) as f64).sqrt();
        let diameter_m = (a * c_norm) / std::f64::consts::PI;

        // Chiral angle: theta = arcsin(sqrt(3) * m / (2 * sqrt(n^2 + nm + m^2)))
        let sin_theta = (3.0_f64.sqrt() * m_val as f64) / (2.0 * c_norm.max(1e-12));
        let chiral_angle_rad = sin_theta.clamp(-1.0, 1.0).asin();

        // Electronic classification and bandgap
        let diff = (n_val as i64 - m_val as i64).abs();
        let (character, bandgap_ev) = if n_val == m_val {
            // Strictly metallic armchair
            (CntCharacter::Metallic, 0.0)
        } else if diff % 3 == 0 {
            // Quasi-metallic with curvature-induced gap:
            // Delta E_curv = (gamma0 * a_cc^2 / (8 * d_t^2)) * cos(3 * theta)
            let cos_3theta = (3.0 * chiral_angle_rad).cos().abs();
            let curv_gap = (gamma0_ev * CARBON_BOND_LENGTH_M.powi(2)
                / (8.0 * diameter_m * diameter_m))
                * cos_3theta;
            (CntCharacter::QuasiMetallic, curv_gap)
        } else {
            // Semiconducting: E_g ~ 2 * a_cc * gamma0 / d_t
            let eg = (2.0 * CARBON_BOND_LENGTH_M * gamma0_ev) / diameter_m;
            (CntCharacter::Semiconducting, eg)
        };

        Self {
            n: n_val,
            m: m_val,
            diameter_m,
            chiral_angle_rad,
            character,
            bandgap_ev,
            hopping_gamma0_ev: gamma0_ev,
            lambda_acoustic_300k_m: 1.0e-6, // 1 um
            hbar_omega_op_ev: 0.16,         // 160 meV
        }
    }

    /// Evaluates the 1D subband dispersion $E_p(k_z)$ for subband index $p \in \{1, 2, 3\}$
    /// along the axial wavevector $k_z$ (in $m^{-1}$):
    /// $$E_p(k_z) = \pm \sqrt{\left(\frac{E_{gp}}{2}\right)^2 + (\hbar v_F k_z)^2}$$
    pub fn subband_dispersion_ev(&self, subband_index: u32, kz_m: f64) -> (f64, f64) {
        // Fermi velocity in graphene: v_F = 3 * gamma0 * a_cc / (2 * hbar) ~ 8e5 - 1e6 m/s
        let v_fermi = (3.0 * self.hopping_gamma0_ev * ELEMENTARY_CHARGE * CARBON_BOND_LENGTH_M)
            / (2.0 * phonon_core::H_BAR);

        let subband_gap_ev = match self.character {
            CntCharacter::Metallic => {
                if subband_index <= 1 {
                    0.0
                } else {
                    // Higher subbands in armchair: E_gp = (subband_index - 1) * 3 * a_cc * gamma0 / d_t
                    (subband_index as f64 - 1.0)
                        * (3.0 * CARBON_BOND_LENGTH_M * self.hopping_gamma0_ev)
                        / self.diameter_m
                }
            }
            CntCharacter::QuasiMetallic => {
                if subband_index <= 1 {
                    self.bandgap_ev
                } else {
                    (subband_index as f64) * self.bandgap_ev.max(0.1)
                }
            }
            CntCharacter::Semiconducting => {
                // Ratio of subband edges in semiconducting CNTs: E_g1, E_g2 = 2 * E_g1, E_g3 = 4 * E_g1
                let factor = match subband_index {
                    0 | 1 => 1.0,
                    2 => 2.0,
                    _ => 4.0,
                };
                factor * self.bandgap_ev
            }
        };

        let delta_e = subband_gap_ev / 2.0;
        let kinetic_energy_ev = (phonon_core::H_BAR * v_fermi * kz_m.abs()) / ELEMENTARY_CHARGE;
        let e_conduction = (delta_e * delta_e + kinetic_energy_ev * kinetic_energy_ev).sqrt();
        let e_valence = -e_conduction;

        (e_valence, e_conduction)
    }

    /// Evaluates quasi-ballistic drain current $I_{DS}$ (in Amperes) of the Carbon Nanotube FET
    /// with channel length $L$, gate overdrive $V_{GS} - V_{TH}$, and bias $V_{DS}$ at temperature $T$:
    pub fn evaluate_fet_current(
        &self,
        v_gs: f64,
        v_ds: f64,
        v_th: f64,
        length_m: f64,
        temp_k: f64,
    ) -> f64 {
        if v_ds.abs() < 1e-12 {
            return 0.0;
        }

        let is_neg = v_ds < 0.0;
        let vds_mag = v_ds.abs();
        let temp_safe = temp_k.max(1.0);
        let kb_t_ev = (BOLTZMANN_CONSTANT * temp_safe) / ELEMENTARY_CHARGE;

        // Acoustic phonon mean free path with T^-1 scaling:
        let lambda_ac = self.lambda_acoustic_300k_m * (300.0 / temp_safe);
        // Optical phonon emission mean free path at high bias:
        let lambda_op = 15.0e-9; // ~15 nm

        // Effective mean free path under bias:
        let lambda_eff = if vds_mag > self.hbar_omega_op_ev {
            1.0 / (1.0 / lambda_ac + 1.0 / lambda_op)
        } else {
            lambda_ac
        };

        // Ballistic transmission coefficient: T_eff = lambda / (L + lambda)
        let transmission = (lambda_eff / (length_m + lambda_eff)).clamp(0.01, 1.0);

        match self.character {
            CntCharacter::Metallic => {
                // Metallic CNT: constant 2 conducting channels (4 e^2 / h)
                // In high field, current saturates around I_sat ~ 25 uA per tube due to optical phonon emission
                let g_low_field = 2.0 * QUANTUM_CONDUCTANCE_SI * transmission;
                let i_sat = 25.0e-6; // 25 uA
                let v_sat = i_sat / g_low_field.max(1e-12);
                let ids_mag = i_sat * (vds_mag / (vds_mag.powi(2) + v_sat.powi(2)).sqrt());
                if is_neg {
                    -ids_mag
                } else {
                    ids_mag
                }
            }
            CntCharacter::QuasiMetallic | CntCharacter::Semiconducting => {
                // Field-effect modulation via Landauer subband transmission
                let vov = v_gs - v_th;
                let ideality = 1.1;

                // Ballistic Landauer thermal integration for 1st subband:
                // I = (4 e / h) * k_B T * [ ln(1 + exp((mu_S - E_c)/(k_B T))) - ln(1 + exp((mu_D - E_c)/(k_B T))) ]
                let eta_s = (vov / (ideality * kb_t_ev)).clamp(-30.0, 30.0);
                let eta_d = ((vov - vds_mag) / (ideality * kb_t_ev)).clamp(-30.0, 30.0);

                let log_term_s = (1.0 + eta_s.exp()).ln();

                let log_term_d = (1.0 + eta_d.exp()).ln();

                let delta_log = (log_term_s - log_term_d).max(0.0);
                let ids_ballistic = (4.0 * ELEMENTARY_CHARGE / PLANCK_CONSTANT)
                    * (BOLTZMANN_CONSTANT * temp_safe)
                    * delta_log;

                // Max saturation per tube limited by optical phonon scattering (~20-25 uA)
                let i_sat_op = 20.0e-6;
                let ids_mag = transmission * (ids_ballistic * i_sat_op)
                    / ((ids_ballistic).powi(2) + i_sat_op.powi(2)).sqrt();

                if is_neg {
                    -ids_mag
                } else {
                    ids_mag
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnt_chirality_classification() {
        // (10, 10) Armchair is strictly metallic
        let cnt_armchair = CarbonNanotube::armchair(10);
        assert_eq!(cnt_armchair.character, CntCharacter::Metallic);
        assert_eq!(cnt_armchair.bandgap_ev, 0.0);
        assert!((cnt_armchair.chiral_angle_rad - std::f64::consts::FRAC_PI_6).abs() < 1e-4);

        // (10, 0) Zigzag has (10 - 0) % 3 = 1 -> Semiconducting
        let cnt_zigzag_semi = CarbonNanotube::zigzag(10);
        assert_eq!(cnt_zigzag_semi.character, CntCharacter::Semiconducting);
        assert!(cnt_zigzag_semi.bandgap_ev > 0.5); // ~ 1.08 eV for d_t ~ 0.78 nm
        assert_eq!(cnt_zigzag_semi.chiral_angle_rad, 0.0);

        // (9, 0) Zigzag has (9 - 0) % 3 = 0 -> QuasiMetallic with small curvature gap
        let cnt_zigzag_quasi = CarbonNanotube::zigzag(9);
        assert_eq!(cnt_zigzag_quasi.character, CntCharacter::QuasiMetallic);
        assert!(cnt_zigzag_quasi.bandgap_ev > 0.0);
        assert!(cnt_zigzag_quasi.bandgap_ev < 0.1); // Small curvature gap < 100 meV
    }

    #[test]
    fn test_cnt_dispersion_and_transport() {
        let cnt = CarbonNanotube::armchair(10);
        let (ev, ec) = cnt.subband_dispersion_ev(1, 0.0);
        assert_eq!(ev, 0.0);
        assert_eq!(ec, 0.0);

        // Ballistic transport in metallic CNT
        let i_metal = cnt.evaluate_fet_current(1.0, 0.1, 0.0, 100e-9, 300.0);
        assert!(i_metal > 0.0);
        assert!(i_metal < 30e-6); // Below optical phonon saturation

        // Semiconducting CNT gate control
        let semi_cnt = CarbonNanotube::zigzag(10);
        let i_off = semi_cnt.evaluate_fet_current(0.0, 0.5, 0.4, 50e-9, 300.0);
        let i_on = semi_cnt.evaluate_fet_current(1.2, 0.5, 0.4, 50e-9, 300.0);
        assert!(i_on > i_off * 50.0);
    }
}
