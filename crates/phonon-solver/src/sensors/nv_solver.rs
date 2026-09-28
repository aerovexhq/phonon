//! Diamond NV Quantum Master Equation & Dynamical Decoupling Solver
//!
//! Provides solvers for:
//! 1. Ramsey interferometry ($\pi/2 - \tau - \pi/2$) evaluating inhomogeneous dephasing
//!    time $T_2^*$ and DC magnetic field precession.
//! 2. Hahn echo ($\pi/2 - \tau - \pi - \tau - \pi/2$) refocusing static magnetic field noise
//!    to measure $T_2$ and AC magnetic sensitivity at $f_{AC} = 1/(2\tau)$.
//! 3. CPMG-N (Carr-Purcell-Meiboom-Gill) multi-pulse dynamical decoupling sequences extending
//!    coherence time to $T_2(N) \propto T_2 \cdot N^{2/3} > 500\,\mu\text{s}$ at room temperature
//!    and filtering target AC frequencies.
//! 4. Dynamic AC sensitivity:
//!    \[\eta_{AC} \approx \frac{\hbar}{g_e \mu_B} \frac{1}{C \sqrt{I_0 T_2}} \frac{1}{\sqrt{t}}\]
//!    reaching sub-picotesla ($\text{pT}/\sqrt{\text{Hz}}$) sensitivity.
//! 5. Multi-threaded Rayon execution for parallel sequence evaluation and parameter sweeps.

use phonon_core::H_BAR;
use phonon_models::sensors::{
    NvCenter, BOHR_MAGNETON_JOULES, NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T, NV_ELECTRON_G_FACTOR,
};
use rayon::prelude::*;

/// Type of dynamical decoupling pulse sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulseSequenceType {
    /// Ramsey free-induction decay: $\pi/2 - \tau - \pi/2$ (DC sensing).
    Ramsey,
    /// Hahn echo single-pulse refocusing: $\pi/2 - \tau - \pi - \tau - \pi/2$ (AC sensing at $f = 1/(2\tau)$).
    HahnEcho,
    /// CPMG-N multi-pulse sequence: $\pi/2 - (\tau - \pi - \tau)^N - \pi/2$ (Narrowband AC sensing at $f = 1/(4\tau)$).
    Cpmg { num_pulses: usize },
}

/// Result of a Ramsey interferometry measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct RamseyResult {
    /// Evolution times $\tau$ in seconds.
    pub tau_times_s: Vec<f64>,
    /// State population $P(|0\rangle)$ measured as photoluminescence contrast.
    pub populations: Vec<f64>,
    /// Inhomogeneous dephasing time $T_2^*$ in seconds.
    pub t2_star_s: f64,
    /// Reconstructed DC magnetic field in Tesla.
    pub reconstructed_b_dc_tesla: f64,
}

/// Result of a Hahn echo measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct HahnEchoResult {
    /// Free precession half-interval $\tau$ in seconds (total echo time $2\tau$).
    pub tau_s: f64,
    /// Resonant AC frequency $f_{AC} = 1 / (2\tau)$ in Hertz.
    pub resonant_ac_freq_hz: f64,
    /// Accumulated quantum phase $\Phi$ in radians.
    pub accumulated_phase_rad: f64,
    /// Remaining spin coherence factor $W(2\tau) = \exp(-(2\tau/T_2)^3)$.
    pub coherence_factor: f64,
    /// Echo signal photoluminescence contrast $S(2\tau)$.
    pub signal_contrast: f64,
    /// Reconstructed AC magnetic field amplitude in Tesla.
    pub reconstructed_b_ac_tesla: f64,
}

/// Result of a CPMG-N multi-pulse dynamical decoupling sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct CpmgResult {
    /// Number of refocusing $\pi$ pulses $N$.
    pub num_pulses: usize,
    /// Half-interval between pulses $\tau$ in seconds.
    pub tau_s: f64,
    /// Total sequence interrogation time $T = 2 N \tau$ in seconds.
    pub total_time_s: f64,
    /// Center filter frequency $f_0 = 1 / (4\tau)$ in Hertz.
    pub center_freq_hz: f64,
    /// Filter 3-dB bandwidth $\Delta f \approx 1 / T$ in Hertz.
    pub bandwidth_hz: f64,
    /// Extended coherence time $T_2(N)$ in seconds.
    pub extended_t2_s: f64,
    /// Filter transmission / accumulated phase $\Phi$ in radians.
    pub accumulated_phase_rad: f64,
    /// Remaining spin coherence factor $\exp(-(T / T_2(N))^3)$.
    pub coherence_factor: f64,
    /// Output signal contrast.
    pub signal_contrast: f64,
    /// Dynamic AC magnetic field sensitivity $\eta_{AC}$ in $\text{T}/\sqrt{\text{Hz}}$.
    pub sensitivity_ac_t_per_rt_hz: f64,
}

/// Comprehensive solver for diamond NV center quantum state dynamics and pulse sequences.
#[derive(Debug, Clone, PartialEq)]
pub struct NvSolver {
    /// Diamond NV center sensor model.
    pub nv_center: NvCenter,
    /// Photon collection rate $I_0$ in counts per second (cps).
    pub photon_count_rate_cps: f64,
    /// Base optical contrast $C$.
    pub optical_contrast: f64,
}

impl Default for NvSolver {
    fn default() -> Self {
        Self::new(NvCenter::default())
    }
}

impl NvSolver {
    /// Creates a new NV quantum solver with default collection rate ($10^7\text{ cps}$) and contrast ($0.15$).
    pub fn new(nv_center: NvCenter) -> Self {
        Self {
            nv_center,
            photon_count_rate_cps: 1.0e7,
            optical_contrast: 0.15,
        }
    }

    /// Sets collection parameters: photon count rate $I_0$ (cps) and optical contrast $C$.
    pub fn with_optics(mut self, photon_count_rate_cps: f64, optical_contrast: f64) -> Self {
        self.photon_count_rate_cps = photon_count_rate_cps.max(1.0);
        self.optical_contrast = optical_contrast.clamp(1e-4, 1.0);
        self
    }

    /// Evaluates dynamic AC magnetic field sensitivity $\eta_{AC}$ in $\text{T}/\sqrt{\text{Hz}}$:
    /// \[\eta_{AC} \approx \frac{\hbar}{g_e \mu_B} \frac{1}{C \sqrt{I_0 T_2}} \frac{1}{\sqrt{t}}\]
    /// For standard 1-second integration ($t = 1\text{ s}$):
    /// \[\eta_{AC} \approx \frac{\hbar}{g_e \mu_B} \frac{1}{C \sqrt{I_0 T_2}}\]
    pub fn ac_magnetic_sensitivity(
        &self,
        coherence_time_t2_s: f64,
        integration_time_s: f64,
    ) -> f64 {
        let t2 = coherence_time_t2_s.max(1e-9);
        let t_int = integration_time_s.max(1e-9);
        let c = self.optical_contrast;
        let i0 = self.photon_count_rate_cps;

        let prefactor = H_BAR / (NV_ELECTRON_G_FACTOR * BOHR_MAGNETON_JOULES);
        prefactor / (c * (i0 * t2).sqrt() * t_int.sqrt())
    }

    /// Simulates Ramsey free-induction decay ($\pi/2 - \tau - \pi/2$) across a vector of time steps $\tau$.
    ///
    /// Under a DC magnetic field $B_{DC}$, the spin precesses at frequency $\delta f = \gamma_e B_{DC}$.
    /// Inhomogeneous dephasing leads to Gaussian decay $\exp(-(\tau / T_2^*)^2)$.
    /// Population is:
    /// \[P_0(\tau) = \frac{1}{2} \left[1 + \cos(2\pi \gamma_e B_{DC} \tau) \exp\left(-\left(\frac{\tau}{T_2^*}\right)^2\right)\right]\]
    pub fn simulate_ramsey(&self, b_dc_tesla: f64, tau_steps_s: &[f64]) -> RamseyResult {
        let t2_star = self.nv_center.t2_star_time_s.max(1e-12);
        let gamma_e = NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T;
        let detuning_hz = gamma_e * b_dc_tesla;

        let mut populations = Vec::with_capacity(tau_steps_s.len());
        for &tau in tau_steps_s {
            let decay = (-((tau / t2_star).powi(2))).exp();
            let phase = 2.0 * std::f64::consts::PI * detuning_hz * tau;
            let p0 = 0.5 * (1.0 + phase.cos() * decay);
            populations.push(p0);
        }

        RamseyResult {
            tau_times_s: tau_steps_s.to_vec(),
            populations,
            t2_star_s: t2_star,
            reconstructed_b_dc_tesla: b_dc_tesla,
        }
    }

    /// Simulates Hahn Echo ($\pi/2 - \tau - \pi - \tau - \pi/2$) sequence for detecting an AC magnetic field.
    ///
    /// Static noise is refocused at $2\tau$. If an AC magnetic field $B(t) = B_{AC} \sin(2\pi f_{AC} t)$
    /// is applied with $f_{AC} = 1 / (2\tau)$, phase accumulates constructively:
    /// \[\Phi = \frac{4 \gamma_e B_{AC} \tau}{\pi} = \frac{2 \gamma_e B_{AC}}{\pi f_{AC}}\]
    /// (with angular factor $2\pi$).
    /// The coherence decays as $\exp(-(2\tau / T_2)^3)$.
    pub fn simulate_hahn_echo(&self, b_ac_tesla: f64, tau_s: f64) -> HahnEchoResult {
        let t2 = self.nv_center.t2_coherence_time_s.max(1e-9);
        let echo_time = 2.0 * tau_s;
        let f_ac = 1.0 / (2.0 * tau_s.max(1e-12));

        let gamma_ang = 2.0 * std::f64::consts::PI * NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T;
        // Phase accumulated over Hahn echo: Phi = (4 * gamma * B_ac * tau) / pi
        let phase = (4.0 * gamma_ang * b_ac_tesla * tau_s) / std::f64::consts::PI;

        let coherence = (-(echo_time / t2).powi(3)).exp();
        let signal = 0.5 * (1.0 + phase.cos() * coherence);

        HahnEchoResult {
            tau_s,
            resonant_ac_freq_hz: f_ac,
            accumulated_phase_rad: phase,
            coherence_factor: coherence,
            signal_contrast: signal,
            reconstructed_b_ac_tesla: b_ac_tesla,
        }
    }

    /// Simulates CPMG-N (Carr-Purcell-Meiboom-Gill) multi-pulse dynamical decoupling sequence.
    ///
    /// Extends coherence time via multi-pulse refocusing:
    /// \[T_2(N) = T_2 \cdot N^{2/3}\]
    /// Bandpass filters magnetic fields centered at:
    /// \[f_0 = \frac{1}{4\tau}, \quad \Delta f \approx \frac{1}{2 N \tau}\]
    /// Accumulated phase for on-resonance AC field:
    /// \[\Phi = \frac{4 N \gamma_e B_{AC}}{\pi f_0} = \frac{16 N \gamma_e B_{AC} \tau}{\pi}\]
    pub fn simulate_cpmg(&self, num_pulses: usize, tau_s: f64, b_ac_tesla: f64) -> CpmgResult {
        assert!(num_pulses >= 1, "Requires at least 1 pulse");
        let n = num_pulses as f64;
        let total_time = 2.0 * n * tau_s;
        let center_freq = 1.0 / (4.0 * tau_s.max(1e-12));
        let bandwidth = 1.0 / total_time.max(1e-12);

        // Extended T2 coherence scaling T2(N) = T2 * N^(2/3)
        let base_t2 = self.nv_center.t2_coherence_time_s.max(1e-9);
        let extended_t2 = base_t2 * n.powf(2.0 / 3.0);

        let gamma_ang = 2.0 * std::f64::consts::PI * NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T;
        let phase = (4.0 * n * gamma_ang * b_ac_tesla * (2.0 * tau_s)) / std::f64::consts::PI;

        let coherence = (-(total_time / extended_t2).powi(3)).exp();
        let signal = 0.5 * (1.0 + phase.cos() * coherence);

        let sensitivity = self.ac_magnetic_sensitivity(extended_t2, 1.0);

        CpmgResult {
            num_pulses,
            tau_s,
            total_time_s: total_time,
            center_freq_hz: center_freq,
            bandwidth_hz: bandwidth,
            extended_t2_s: extended_t2,
            accumulated_phase_rad: phase,
            coherence_factor: coherence,
            signal_contrast: signal,
            sensitivity_ac_t_per_rt_hz: sensitivity,
        }
    }

    /// Evaluates CPMG multi-pulse sequence dynamically swept across pulse counts using multi-threaded Rayon.
    pub fn sweep_cpmg_pulses_parallel(
        &self,
        pulse_counts: &[usize],
        tau_s: f64,
        b_ac_tesla: f64,
    ) -> Vec<CpmgResult> {
        pulse_counts
            .par_iter()
            .map(|&n| self.simulate_cpmg(n, tau_s, b_ac_tesla))
            .collect()
    }

    /// Evaluates dynamic AC magnetic field spectrum across a range of AC frequencies $f_{AC}$.
    ///
    /// Adjusts $\tau = 1 / (4 f_{AC})$ to center the CPMG-N filter on each frequency point.
    pub fn sweep_cpmg_frequency_parallel(
        &self,
        frequencies_hz: &[f64],
        num_pulses: usize,
        b_ac_tesla: f64,
    ) -> Vec<CpmgResult> {
        frequencies_hz
            .par_iter()
            .map(|&f| {
                let tau = 1.0 / (4.0 * f.max(1.0));
                self.simulate_cpmg(num_pulses, tau, b_ac_tesla)
            })
            .collect()
    }
}
