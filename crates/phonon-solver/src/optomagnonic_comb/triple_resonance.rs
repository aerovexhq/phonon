#![deny(unsafe_code)]

//! Triple-Resonance Brillouin and Optomagnonic Polariton Coupling Engine.
//!
//! Models tripartite interaction among an optical whispering-gallery mode (WGM) photon,
//! a collective magnetic spin excitation (magnon), and an acoustic phonon.
//!
//! Conservation equations:
//! - Energy: omega_pump - omega_Stokes = omega_magnon + omega_phonon
//! - Tri-modal detuning error: Delta_omega = |(omega_1 - omega_2) - (omega_m + omega_p)|
//! - Triple-resonance criterion: Delta_omega <= kappa_opt / 2
//! - Polariton gap at avoided crossing: Delta_E = 2 * sqrt(g_om^2 + g_am^2)

use std::f64::consts::PI;

/// Parameters defining the triple-resonance cavity optomagnonic system.
#[derive(Debug, Clone, PartialEq)]
pub struct TripleResonanceParams {
    /// Optical pump frequency in THz (e.g. 193.4 THz / 1550 nm telecommunications band).
    pub optical_freq_thz: f64,
    /// Optical Stokes mode frequency in THz (default 193.375 THz).
    pub stokes_optical_freq_thz: f64,
    /// Magnon resonance frequency in GHz (e.g. 10.0 GHz in YIG sphere/disk).
    pub magnon_freq_ghz: f64,
    /// Acoustic phonon frequency in GHz (e.g. 15.0 GHz bulk/surface acoustic wave).
    pub acoustic_phonon_freq_ghz: f64,
    /// Optical cavity linewidth kappa_opt in MHz (Q ~ 4e6 -> 50.0 MHz).
    pub optical_linewidth_mhz: f64,
    /// Magnon resonance linewidth kappa_mag in MHz (e.g. 2.0 MHz).
    pub magnon_linewidth_mhz: f64,
    /// Acoustic phonon linewidth kappa_phon in MHz (e.g. 1.0 MHz).
    pub phonon_linewidth_mhz: f64,
    /// Optomagnonic coupling rate g_om in kHz (default ~50.0 kHz).
    pub optomagnonic_coupling_khz: f64,
    /// Acoustomagnonic magnetoelastic coupling rate g_am in MHz (default ~20.0 MHz).
    pub acoustomagnonic_coupling_mhz: f64,
}

impl Default for TripleResonanceParams {
    fn default() -> Self {
        Self {
            optical_freq_thz: 193.4,
            stokes_optical_freq_thz: 193.375, // Exactly 25 GHz separation
            magnon_freq_ghz: 10.0,
            acoustic_phonon_freq_ghz: 15.0,
            optical_linewidth_mhz: 50.0,
            magnon_linewidth_mhz: 2.0,
            phonon_linewidth_mhz: 1.0,
            optomagnonic_coupling_khz: 50.0,
            acoustomagnonic_coupling_mhz: 20.0,
        }
    }
}

/// Representation of a single polariton eigenbranch.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonBranch {
    /// Name identifier: Lower Polariton (LP), Middle Polariton (MP), or Upper Polariton (UP).
    pub name: String,
    /// Eigenfrequency shift relative to center resonance in MHz.
    pub eigenfrequency_shift_mhz: f64,
    /// Absolute hybridized frequency in GHz.
    pub absolute_freq_ghz: f64,
    /// Hybridization fraction of optical photon |c_opt|^2.
    pub photon_fraction: f64,
    /// Hybridization fraction of spin magnon |c_mag|^2.
    pub magnon_fraction: f64,
    /// Hybridization fraction of acoustic phonon |c_phon|^2.
    pub phonon_fraction: f64,
    /// Hybridized polariton linewidth in MHz.
    pub effective_linewidth_mhz: f64,
}

/// Point on the avoided crossing dispersion curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvoidedCrossingPoint {
    /// Detuning of optical mode from magnon-phonon sum in MHz.
    pub detuning_mhz: f64,
    /// Lower polariton branch frequency shift in MHz.
    pub lower_branch_mhz: f64,
    /// Middle polariton branch frequency shift in MHz.
    pub middle_branch_mhz: f64,
    /// Upper polariton branch frequency shift in MHz.
    pub upper_branch_mhz: f64,
}

/// Resulting metrics and polariton eigensystem from the triple-resonance solver.
#[derive(Debug, Clone, PartialEq)]
pub struct TripleResonanceResult {
    /// Optical pump frequency in THz.
    pub optical_pump_thz: f64,
    /// Stokes optical frequency in THz.
    pub optical_stokes_thz: f64,
    /// Optical mode frequency separation in GHz: f_pump - f_Stokes.
    pub optical_separation_ghz: f64,
    /// Combined magnon + phonon excitation frequency in GHz.
    pub sum_excitation_ghz: f64,
    /// Tri-modal detuning error in MHz: Delta_omega = |(f_pump - f_Stokes) - (f_m + f_p)|.
    pub detuning_error_mhz: f64,
    /// Allowable optical half-linewidth threshold kappa_opt / 2 in MHz.
    pub optical_half_linewidth_mhz: f64,
    /// Whether triple-resonance condition is strictly met: Delta_omega <= kappa_opt / 2.
    pub is_triple_resonant: bool,
    /// Resonance margin in MHz: kappa_opt / 2 - Delta_omega.
    pub resonance_margin_mhz: f64,
    /// Avoided crossing polariton splitting gap in MHz: 2 * sqrt(g_om^2 + g_am^2).
    pub avoided_crossing_gap_mhz: f64,
    /// Optomagnonic cooperativity C_om = 4 * g_om^2 / (kappa_opt * kappa_mag).
    pub cooperativity_om: f64,
    /// Acoustomagnonic cooperativity C_am = 4 * g_am^2 / (kappa_mag * kappa_phon).
    pub cooperativity_am: f64,
    /// Tripartite polariton branches [Lower, Middle, Upper].
    pub polariton_branches: [PolaritonBranch; 3],
}

/// Analytical and eigensystem solver for cavity optomagnonic triple resonance.
pub struct TripleResonanceSolver;

impl TripleResonanceSolver {
    /// Solves the tripartite optomagnonic-acoustic resonance condition and polariton branches.
    pub fn solve(params: &TripleResonanceParams) -> TripleResonanceResult {
        // Optical mode separation in GHz
        let opt_sep_ghz = (params.optical_freq_thz - params.stokes_optical_freq_thz) * 1000.0;
        let sum_excitation_ghz = params.magnon_freq_ghz + params.acoustic_phonon_freq_ghz;

        // Tri-modal detuning error in MHz
        let detuning_error_ghz = (opt_sep_ghz - sum_excitation_ghz).abs();
        let detuning_error_mhz = detuning_error_ghz * 1000.0;

        let optical_half_linewidth_mhz = params.optical_linewidth_mhz / 2.0;
        let is_triple_resonant = detuning_error_mhz <= optical_half_linewidth_mhz;
        let resonance_margin_mhz = optical_half_linewidth_mhz - detuning_error_mhz;

        // Couplings in MHz
        let g_om_mhz = params.optomagnonic_coupling_khz / 1000.0;
        let g_am_mhz = params.acoustomagnonic_coupling_mhz;

        // Avoided crossing gap: 2 * sqrt(g_om^2 + g_am^2)
        let total_coupling_mhz = (g_om_mhz * g_om_mhz + g_am_mhz * g_am_mhz).sqrt();
        let avoided_crossing_gap_mhz = 2.0 * total_coupling_mhz;

        // Cooperativities
        let cooperativity_om = (4.0 * g_om_mhz * g_om_mhz)
            / (params.optical_linewidth_mhz * params.magnon_linewidth_mhz).max(1e-12);
        let cooperativity_am = (4.0 * g_am_mhz * g_am_mhz)
            / (params.magnon_linewidth_mhz * params.phonon_linewidth_mhz).max(1e-12);

        // Evaluate tripartite polariton eigensystem
        // Hamiltonian in basis [photon, magnon, phonon]:
        // H = [[delta_opt, g_om, 0],
        //      [g_om, 0, g_am],
        //      [0, g_am, 0]]
        let delta_opt_mhz = (opt_sep_ghz - sum_excitation_ghz) * 1000.0;
        let branches = Self::solve_eigensystem(delta_opt_mhz, g_om_mhz, g_am_mhz, params);

        TripleResonanceResult {
            optical_pump_thz: params.optical_freq_thz,
            optical_stokes_thz: params.stokes_optical_freq_thz,
            optical_separation_ghz: opt_sep_ghz,
            sum_excitation_ghz,
            detuning_error_mhz,
            optical_half_linewidth_mhz,
            is_triple_resonant,
            resonance_margin_mhz,
            avoided_crossing_gap_mhz,
            cooperativity_om,
            cooperativity_am,
            polariton_branches: branches,
        }
    }

    /// Solves the 3x3 symmetric tridiagonal eigensystem for arbitrary optical detuning.
    pub fn solve_eigensystem(
        delta_mhz: f64,
        g_om_mhz: f64,
        g_am_mhz: f64,
        params: &TripleResonanceParams,
    ) -> [PolaritonBranch; 3] {
        // Characteristic equation:
        // det(lambda*I - H) = 0
        // | lambda - delta    -g_om          0     |
        // | -g_om             lambda        -g_am  | = 0
        // | 0                -g_am          lambda |
        //
        // lambda * (lambda * (lambda - delta) - g_am^2) - g_om^2 * lambda = 0
        // lambda * [lambda^2 - delta * lambda - (g_om^2 + g_am^2)] = 0

        let omega_sq = g_om_mhz * g_om_mhz + g_am_mhz * g_am_mhz;

        // Roots of lambda^2 - delta * lambda - omega_sq = 0:
        // lambda = (delta +/- sqrt(delta^2 + 4 * omega_sq)) / 2
        let disc = (delta_mhz * delta_mhz + 4.0 * omega_sq).sqrt();
        let lambda_lower = (delta_mhz - disc) / 2.0;
        let lambda_middle = 0.0;
        let lambda_upper = (delta_mhz + disc) / 2.0;

        let base_freq_ghz = params.magnon_freq_ghz + params.acoustic_phonon_freq_ghz;

        // Helper to compute eigenvectors [c_o, c_m, c_p] for an eigenvalue lambda:
        // (lambda - delta) * c_o - g_om * c_m = 0  => c_m = ((lambda - delta) / g_om) * c_o
        // -g_am * c_m + lambda * c_p = 0           => c_p = (g_am / lambda) * c_m
        let calc_branch = |name: &str, lambda: f64| -> PolaritonBranch {
            let (frac_o, frac_m, frac_p) = if lambda.abs() < 1e-9 {
                // lambda = 0:
                // -delta * c_o - g_om * c_m = 0
                // -g_am * c_m = 0 => c_m = 0
                // If c_m = 0 and g_om != 0:
                // g_om * c_o + g_am * c_p = 0 => c_o = -g_am/g_om * c_p
                let denom = (g_om_mhz * g_om_mhz + g_am_mhz * g_am_mhz).max(1e-12);
                let p_frac = (g_om_mhz * g_om_mhz) / denom;
                let o_frac = (g_am_mhz * g_am_mhz) / denom;
                (o_frac, 0.0, p_frac)
            } else {
                let g_om = g_om_mhz.max(1e-6);
                let co = 1.0;
                let cm = (lambda - delta_mhz) / g_om;
                let cp = (g_am_mhz / lambda) * cm;
                let norm = (co * co + cm * cm + cp * cp).sqrt().max(1e-12);
                let no = co / norm;
                let nm = cm / norm;
                let np = cp / norm;
                (no * no, nm * nm, np * np)
            };

            // Effective linewidth: sum of fractional linewidths
            let eff_lw = frac_o * params.optical_linewidth_mhz
                + frac_m * params.magnon_linewidth_mhz
                + frac_p * params.phonon_linewidth_mhz;

            PolaritonBranch {
                name: name.to_string(),
                eigenfrequency_shift_mhz: lambda,
                absolute_freq_ghz: base_freq_ghz + lambda / 1000.0,
                photon_fraction: frac_o,
                magnon_fraction: frac_m,
                phonon_fraction: frac_p,
                effective_linewidth_mhz: eff_lw,
            }
        };

        [
            calc_branch("Lower Polariton", lambda_lower),
            calc_branch("Middle Polariton", lambda_middle),
            calc_branch("Upper Polariton", lambda_upper),
        ]
    }

    /// Computes the avoided crossing dispersion curves across a detuning sweep.
    pub fn compute_avoided_crossing_curve(
        params: &TripleResonanceParams,
        points: usize,
    ) -> Vec<AvoidedCrossingPoint> {
        let span_mhz = 50.0;
        let mut curve = Vec::with_capacity(points);

        let g_om_mhz = params.optomagnonic_coupling_khz / 1000.0;
        let g_am_mhz = params.acoustomagnonic_coupling_mhz;

        for i in 0..points {
            let frac = i as f64 / (points - 1).max(1) as f64;
            let detuning = -span_mhz + 2.0 * span_mhz * frac;
            let branches = Self::solve_eigensystem(detuning, g_om_mhz, g_am_mhz, params);

            curve.push(AvoidedCrossingPoint {
                detuning_mhz: detuning,
                lower_branch_mhz: branches[0].eigenfrequency_shift_mhz,
                middle_branch_mhz: branches[1].eigenfrequency_shift_mhz,
                upper_branch_mhz: branches[2].eigenfrequency_shift_mhz,
            });
        }

        curve
    }
}
