#![deny(unsafe_code)]

//! Sub-Diffraction Deep-Nanoscale Plasmonic Waveguide Routing & Logic Gates.
//!
//! Solves power transfer across deep-nanoscale directional couplers, sharp sub-micron
//! waveguide bends, and optical transistor switching logic.

use phonon_models::quantum_plasmonics::SinglePhotonTransistor;
use std::f64::consts::PI;

/// Directional Coupler Power Routing Report.
#[derive(Debug, Clone, Copy)]
pub struct PlasmonicCouplerReport {
    /// Physical coupling length in meters.
    pub length: f64,
    /// Coupling coefficient $\kappa_c$ in m^-1.
    pub coupling_coefficient: f64,
    /// Full crossover transfer length $L_c = \pi / (2 \kappa_c)$ in meters.
    pub crossover_length: f64,
    /// Power fraction emerging from through port $P_{through} = \cos^2(\kappa_c L)$.
    pub through_power_fraction: f64,
    /// Power fraction emerging from cross port $P_{cross} = \sin^2(\kappa_c L)$.
    pub cross_power_fraction: f64,
}

/// Evaluates power transfer in a parallel plasmonic slot waveguide directional coupler.
pub fn evaluate_plasmonic_directional_coupler(
    gap_spacing: f64,
    waveguide_length: f64,
    decay_length: f64,
) -> PlasmonicCouplerReport {
    // Evanescent coupling decay across the dielectric barrier
    let kappa_0 = 1.0e6; // 1 um^-1 baseline at zero gap
    let kappa_c = kappa_0 * (-gap_spacing / decay_length.max(1e-9)).exp();

    let l_c = if kappa_c > 1e-12 {
        PI / (2.0 * kappa_c)
    } else {
        1.0e-3
    };

    let phase = kappa_c * waveguide_length;
    let p_through = phase.cos().powi(2);
    let p_cross = phase.sin().powi(2);

    PlasmonicCouplerReport {
        length: waveguide_length,
        coupling_coefficient: kappa_c,
        crossover_length: l_c,
        through_power_fraction: p_through,
        cross_power_fraction: p_cross,
    }
}

/// Sharp Sub-Micron Waveguide Bend Transmission Report.
#[derive(Debug, Clone, Copy)]
pub struct WaveguideBendReport {
    /// Bend radius $R_{bend}$ in meters.
    pub bend_radius: f64,
    /// Bend angle $\theta$ in radians.
    pub bend_angle: f64,
    /// Effective mode index $n_{eff}$.
    pub effective_index: f64,
    /// Power transmission fraction $T_{bend} \in [0, 1]$.
    pub transmission: f64,
    /// Bending loss in decibels.
    pub loss_db: f64,
}

/// Evaluates transmission around a sharp sub-micron plasmonic waveguide bend.
pub fn evaluate_waveguide_bend(
    bend_radius: f64,
    bend_angle: f64,
    effective_index: f64,
    propagation_length: f64,
) -> WaveguideBendReport {
    // Arc length of the bend
    let arc_length = bend_radius * bend_angle;

    // Ohmic attenuation: exp(-arc_length / L_spp)
    let ohmic_trans = (-arc_length / propagation_length.max(1e-9)).exp();

    // Radiation loss: suppressed exponentially by high effective index n_eff and k0 * R_bend
    let k0 = 8.0e6; // nominal optical wavenumber in vacuum
    let rad_loss = (-2.0 * effective_index * k0 * bend_radius).exp();
    let rad_trans = (1.0 - rad_loss).clamp(0.0, 1.0);

    let total_trans = (ohmic_trans * rad_trans).clamp(0.01, 1.0);
    let loss_db = -10.0 * total_trans.log10();

    WaveguideBendReport {
        bend_radius,
        bend_angle,
        effective_index,
        transmission: total_trans,
        loss_db,
    }
}

/// Single-Photon Transistor Logic Evaluation Report.
#[derive(Debug, Clone, Copy)]
pub struct TransistorLogicReport {
    /// Unpumped probe transmission $T_0$ (closed / OFF state).
    pub transmission_off: f64,
    /// Saturated probe transmission $T_1$ (open / ON state).
    pub transmission_on: f64,
    /// Optical switching contrast in decibels.
    pub switching_contrast_db: f64,
    /// Transmitted probe photon count per gate photon (optical gain).
    pub optical_gain: f64,
    /// Switching energy per bit in Joules.
    pub switching_energy_joules: f64,
}

/// Evaluates digital all-optical switching metrics for a single-photon plasmonic transistor.
pub fn evaluate_transistor_logic(
    transistor: &SinglePhotonTransistor,
    probe_flux_photons_per_sec: f64,
) -> TransistorLogicReport {
    let t0 = transistor.unpumped_transmission();
    let t1 = transistor.saturated_transmission();
    let contrast_db = transistor.switching_contrast_db();
    let gain = transistor.optical_transistor_gain(probe_flux_photons_per_sec);

    // Switching energy = energy of a single gate photon: hbar * omega_0
    let e_switch = phonon_models::quantum_plasmonics::HBAR * transistor.emitter.angular_frequency();

    TransistorLogicReport {
        transmission_off: t0,
        transmission_on: t1,
        switching_contrast_db: contrast_db,
        optical_gain: gain,
        switching_energy_joules: e_switch,
    }
}
