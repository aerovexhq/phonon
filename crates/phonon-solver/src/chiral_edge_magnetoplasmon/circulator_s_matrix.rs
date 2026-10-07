#![deny(unsafe_code)]

//! 3-Port Cyclic Circulator Scattering Matrix ($S$-Matrix) Solver.
//!
//! Models non-reciprocal acoustic edge-magnetoplasmon circulators exhibiting
//! forward transmission with insertion loss <= 0.5 dB, reverse isolation >= 35 dB,
//! and return loss >= 25 dB with cyclic symmetry (1 -> 2 -> 3 -> 1).

use std::f64::consts::PI;

/// Parameters defining the 3-port chiral EMP circulator.
#[derive(Debug, Clone, PartialEq)]
pub struct EmpCirculatorParams {
    /// Operating center RF frequency in GHz (e.g. 3.0 GHz).
    pub center_freq_ghz: f64,
    /// Operating 3-dB circulation bandwidth in MHz (e.g. 350.0 MHz).
    pub bandwidth_mhz: f64,
    /// Forward insertion loss in dB at center frequency (must be <= 0.50 dB, default 0.35 dB).
    pub insertion_loss_db: f64,
    /// Reverse isolation in dB at center frequency (must be >= 35.0 dB, default 38.5 dB).
    pub reverse_isolation_db: f64,
    /// Return loss in dB at center frequency (must be >= 25.0 dB, default 26.5 dB).
    pub return_loss_db: f64,
    /// Unloaded acoustic resonance quality factor Q_0.
    pub quality_factor: f64,
}

impl Default for EmpCirculatorParams {
    fn default() -> Self {
        Self {
            center_freq_ghz: 3.0,
            bandwidth_mhz: 350.0,
            insertion_loss_db: 0.35,
            reverse_isolation_db: 38.5,
            return_loss_db: 26.5,
            quality_factor: 1200.0,
        }
    }
}

/// A 3x3 Complex Scattering Matrix for a 3-port network.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpSMatrix3x3 {
    /// S11, S12, S13 magnitude and phase
    pub s11_mag: f64,
    pub s11_phase_rad: f64,
    pub s12_mag: f64,
    pub s12_phase_rad: f64,
    pub s13_mag: f64,
    pub s13_phase_rad: f64,

    /// S21, S22, S23
    pub s21_mag: f64,
    pub s21_phase_rad: f64,
    pub s22_mag: f64,
    pub s22_phase_rad: f64,
    pub s23_mag: f64,
    pub s23_phase_rad: f64,

    /// S31, S32, S33
    pub s31_mag: f64,
    pub s31_phase_rad: f64,
    pub s32_mag: f64,
    pub s32_phase_rad: f64,
    pub s33_mag: f64,
    pub s33_phase_rad: f64,
}

impl EmpSMatrix3x3 {
    /// Computes the Frobenius difference norm ||S^\dagger * S - I||_F to test matrix unitarity.
    ///
    /// For a passive lossy circulator with 0.35 dB insertion loss, ||S^\dagger * S - I||_F is ~ 0.17.
    pub fn unitarity_error(&self) -> f64 {
        // Form column vectors as [c1, c2, c3]
        // Element (i, j) of S^\dagger * S is col_i^\dagger * col_j
        let col = [
            [(self.s11_mag, self.s11_phase_rad), (self.s21_mag, self.s21_phase_rad), (self.s31_mag, self.s31_phase_rad)],
            [(self.s12_mag, self.s12_phase_rad), (self.s22_mag, self.s22_phase_rad), (self.s32_mag, self.s32_phase_rad)],
            [(self.s13_mag, self.s13_phase_rad), (self.s23_mag, self.s23_phase_rad), (self.s33_mag, self.s33_phase_rad)],
        ];

        let mut sum_sq = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                // Dot product col[i]^\dagger * col[j]
                let mut re = 0.0;
                let mut im = 0.0;
                for k in 0..3 {
                    let (m_i, p_i) = col[i][k];
                    let (m_j, p_j) = col[j][k];
                    let phase_diff = p_j - p_i;
                    re += m_i * m_j * phase_diff.cos();
                    im += m_i * m_j * phase_diff.sin();
                }

                let target_re = if i == j { 1.0 } else { 0.0 };
                let diff_re = re - target_re;
                let diff_im = im;
                sum_sq += diff_re * diff_re + diff_im * diff_im;
            }
        }

        sum_sq.sqrt()
    }
}

/// A spectrum sample point recording S-parameters across frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpSpectrumPoint {
    pub freq_ghz: f64,
    pub s21_db: f64,
    pub s12_db: f64,
    pub s11_db: f64,
    pub s32_db: f64,
    pub s23_db: f64,
}

/// Engine for evaluating 3-port cyclic EMP circulator scattering parameters.
#[derive(Debug, Clone)]
pub struct EmpCirculator {
    pub params: EmpCirculatorParams,
}

impl EmpCirculator {
    /// Creates a new circulator engine with the given parameters.
    pub fn new(params: EmpCirculatorParams) -> Self {
        Self { params }
    }

    /// Evaluates the 3x3 S-matrix at the given frequency in GHz.
    pub fn evaluate_s_matrix(&self, freq_ghz: f64) -> EmpSMatrix3x3 {
        let f0 = self.params.center_freq_ghz;
        let bw_ghz = self.params.bandwidth_mhz * 1.0e-3;
        let delta_f = (freq_ghz - f0).abs();
        let normalized_detuning = (delta_f / (bw_ghz * 0.5)).max(0.0);

        // Baseline magnitudes at center frequency from parameter dB specifications
        let s21_center_mag = 10.0_f64.powf(-self.params.insertion_loss_db / 20.0);
        let s12_center_mag = 10.0_f64.powf(-self.params.reverse_isolation_db / 20.0);
        let s11_center_mag = 10.0_f64.powf(-self.params.return_loss_db / 20.0);

        // Frequency-dependent roll-off (Lorentzian bandpass resonance)
        let roll_off = 1.0 / (1.0 + normalized_detuning * normalized_detuning).sqrt();
        let s21_mag = (s21_center_mag * roll_off).clamp(1e-6, 1.0);

        // Reverse leakage increases smoothly away from band center
        let leakage_increase = (normalized_detuning * 0.02).min(0.2);
        let s12_mag = (s12_center_mag + leakage_increase).clamp(1e-6, 0.9);

        // Reflection increases out-of-band as transmission falls
        let s11_mag = (s11_center_mag + (1.0 - roll_off) * 0.9).clamp(1e-6, 0.999);

        // Cyclic symmetry: 1 -> 2, 2 -> 3, 3 -> 1 forward paths
        let s21 = (s21_mag, 0.0);
        let s32 = (s21_mag, -2.0 * PI / 3.0);
        let s13 = (s21_mag, -4.0 * PI / 3.0);

        // Reverse isolated paths: 1 <- 2, 2 <- 3, 3 <- 1
        let s12 = (s12_mag, PI / 2.0);
        let s23 = (s12_mag, PI / 2.0 - 2.0 * PI / 3.0);
        let s31 = (s12_mag, PI / 2.0 - 4.0 * PI / 3.0);

        // Reflection paths on each port
        let s11 = (s11_mag, PI);
        let s22 = (s11_mag, PI);
        let s33 = (s11_mag, PI);

        EmpSMatrix3x3 {
            s11_mag: s11.0,
            s11_phase_rad: s11.1,
            s12_mag: s12.0,
            s12_phase_rad: s12.1,
            s13_mag: s13.0,
            s13_phase_rad: s13.1,

            s21_mag: s21.0,
            s21_phase_rad: s21.1,
            s22_mag: s22.0,
            s22_phase_rad: s22.1,
            s23_mag: s23.0,
            s23_phase_rad: s23.1,

            s31_mag: s31.0,
            s31_phase_rad: s31.1,
            s32_mag: s32.0,
            s32_phase_rad: s32.1,
            s33_mag: s33.0,
            s33_phase_rad: s33.1,
        }
    }

    /// Evaluates forward transmission S21 in dB at center frequency.
    pub fn forward_insertion_loss_db(&self) -> f64 {
        let sm = self.evaluate_s_matrix(self.params.center_freq_ghz);
        -20.0 * sm.s21_mag.log10()
    }

    /// Evaluates reverse isolation S12 in dB at center frequency.
    pub fn reverse_isolation_db(&self) -> f64 {
        let sm = self.evaluate_s_matrix(self.params.center_freq_ghz);
        -20.0 * sm.s12_mag.log10()
    }

    /// Evaluates return loss S11 in dB at center frequency.
    pub fn return_loss_db(&self) -> f64 {
        let sm = self.evaluate_s_matrix(self.params.center_freq_ghz);
        -20.0 * sm.s11_mag.log10()
    }

    /// Computes frequency response curve across span [f_min, f_max].
    pub fn compute_spectrum_sweep(
        &self,
        f_min_ghz: f64,
        f_max_ghz: f64,
        points: usize,
    ) -> Vec<EmpSpectrumPoint> {
        let n = points.max(2);
        let step = (f_max_ghz - f_min_ghz) / (n - 1) as f64;
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            let f = f_min_ghz + i as f64 * step;
            let sm = self.evaluate_s_matrix(f);

            let s21_db = 20.0 * sm.s21_mag.max(1e-6).log10();
            let s12_db = 20.0 * sm.s12_mag.max(1e-6).log10();
            let s11_db = 20.0 * sm.s11_mag.max(1e-6).log10();
            let s32_db = 20.0 * sm.s32_mag.max(1e-6).log10();
            let s23_db = 20.0 * sm.s23_mag.max(1e-6).log10();

            result.push(EmpSpectrumPoint {
                freq_ghz: f,
                s21_db,
                s12_db,
                s11_db,
                s32_db,
                s23_db,
            });
        }

        result
    }
}
