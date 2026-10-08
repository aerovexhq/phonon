#![deny(unsafe_code)]

//! Physical Wiring Realism, Parasitic Extraction (PEX) & Electromagnetic Disturbance (EMI) Engine.
//!
//! Models real-world PCB transmission line effects, trace parasitic resistance, self-inductance,
//! capacitance to ground plane, and high-frequency mutual electromagnetic crosstalk disturbance
//! coupling between adjacent parallel copper traces.

use crate::board_synthesis::trace_router::RoutedPhysicalNet;
use std::f32::consts::PI;

/// Copper physical properties and standard PCB stackup constants.
pub const COPPER_RESISTIVITY_OHM_M: f32 = 1.68e-8; // 1.68e-8 Ohm*m
pub const COPPER_THICKNESS_MM: f32 = 0.035;        // 1 oz copper (35 um)
pub const FR4_RELATIVE_PERMITTIVITY: f32 = 4.4;    // Standard FR4
pub const DIELECTRIC_HEIGHT_MM: f32 = 1.6;         // Standard 1.6mm PCB
pub const VACUUM_PERMITTIVITY: f32 = 8.854e-12;   // F/m
pub const VACUUM_PERMEABILITY: f32 = 1.2566e-6;   // H/m
pub const SPEED_OF_LIGHT_M_S: f32 = 3.0e8;

/// Extracted electrical parasitics for a single physical trace net.
#[derive(Debug, Clone, PartialEq)]
pub struct NetParasitics {
    pub net_name: String,
    pub total_length_mm: f32,
    pub resistance_ohms: f32,
    pub self_inductance_nh: f32,
    pub capacitance_to_gnd_pf: f32,
    pub characteristic_impedance_ohms: f32,
    pub propagation_delay_ps: f32,
}

/// Mutual electromagnetic disturbance coupling between two physically adjacent traces.
#[derive(Debug, Clone, PartialEq)]
pub struct EmiDisturbancePair {
    pub aggressor_net: String,
    pub victim_net: String,
    pub parallel_coupling_length_mm: f32,
    pub separation_distance_mm: f32,
    pub mutual_inductance_nh: f32,
    pub mutual_capacitance_pf: f32,
    /// Estimated induced crosstalk peak noise voltage (in mV) for a standard 1ns CMOS switching edge.
    pub induced_crosstalk_noise_mv: f32,
}

/// Full physical wiring and electromagnetic disturbance report.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalWiringRealismReport {
    pub net_parasitics: Vec<NetParasitics>,
    pub emi_disturbances: Vec<EmiDisturbancePair>,
    pub max_crosstalk_noise_mv: f32,
    pub estimated_ground_bounce_mv: f32,
}

/// Physical Parasitics and Electromagnetic Interference extraction engine.
pub struct PhysicalParasiticsEngine;

impl PhysicalParasiticsEngine {
    /// Extracts parasitic R, L, C and propagation characteristics for each routed net.
    pub fn extract_net_parasitics(net: &RoutedPhysicalNet) -> NetParasitics {
        let length_m = (net.total_length_mm * 1e-3).max(1e-6);
        let width_m = if net.is_power { 0.50e-3 } else { 0.25e-3 };
        let thickness_m = COPPER_THICKNESS_MM * 1e-3;
        let height_m = DIELECTRIC_HEIGHT_MM * 1e-3;

        // 1. DC Resistance: R = rho * L / (W * t)
        let area_m2 = width_m * thickness_m;
        let r_dc = COPPER_RESISTIVITY_OHM_M * (length_m / area_m2);

        // 2. Microstrip Self-Inductance (nH): L = (mu_0 * L / 2pi) * [ln(2L / (W + t)) + 0.5]
        let ln_term = (2.0 * length_m / (width_m + thickness_m)).max(1.01).ln();
        let l_henry = (VACUUM_PERMEABILITY * length_m / (2.0 * PI)) * (ln_term + 0.5);
        let l_nh = l_henry * 1e9;

        // 3. Capacitance to Ground Plane (pF): C = eps_0 * eps_r * W * L / h
        let c_farad = (VACUUM_PERMITTIVITY * FR4_RELATIVE_PERMITTIVITY * width_m * length_m) / height_m;
        let c_pf = c_farad * 1e12;

        // 4. Characteristic Impedance: Z0 = sqrt(L / C)
        let z0 = if c_farad > 0.0 {
            (l_henry / c_farad).sqrt()
        } else {
            50.0
        };

        // 5. Propagation delay: tau = L * sqrt(eps_eff) / c
        let eps_eff = (FR4_RELATIVE_PERMITTIVITY + 1.0) / 2.0;
        let delay_s = (length_m * eps_eff.sqrt()) / SPEED_OF_LIGHT_M_S;
        let delay_ps = delay_s * 1e12;

        NetParasitics {
            net_name: net.net_name.clone(),
            total_length_mm: net.total_length_mm,
            resistance_ohms: r_dc,
            self_inductance_nh: l_nh,
            capacitance_to_gnd_pf: c_pf,
            characteristic_impedance_ohms: z0,
            propagation_delay_ps: delay_ps,
        }
    }

    /// Evaluates electromagnetic disturbance coupling and crosstalk between all adjacent trace pairs.
    pub fn evaluate_board_realism(nets: &[RoutedPhysicalNet]) -> PhysicalWiringRealismReport {
        let mut net_parasitics = Vec::new();
        for net in nets {
            net_parasitics.push(Self::extract_net_parasitics(net));
        }

        let mut emi_disturbances = Vec::new();
        let mut max_crosstalk = 0.0f32;
        let height_mm = DIELECTRIC_HEIGHT_MM;

        // Compare trace segments across distinct nets to detect parallel routing proximity
        let n = nets.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let net_a = &nets[i];
                let net_b = &nets[j];

                for seg_a in &net_a.segments {
                    for seg_b in &net_b.segments {
                        // If parallel horizontal segments
                        let is_parallel_h = (seg_a.start_mm.y - seg_a.end_mm.y).abs() < 1e-3
                            && (seg_b.start_mm.y - seg_b.end_mm.y).abs() < 1e-3;
                        // If parallel vertical segments
                        let is_parallel_v = (seg_a.start_mm.x - seg_a.end_mm.x).abs() < 1e-3
                            && (seg_b.start_mm.x - seg_b.end_mm.x).abs() < 1e-3;

                        if is_parallel_h {
                            let dist = (seg_a.start_mm.y - seg_b.start_mm.y).abs();
                            if dist > 0.1 && dist < 12.0 {
                                // Overlap length
                                let min_x1 = seg_a.start_mm.x.min(seg_a.end_mm.x);
                                let max_x1 = seg_a.start_mm.x.max(seg_a.end_mm.x);
                                let min_x2 = seg_b.start_mm.x.min(seg_b.end_mm.x);
                                let max_x2 = seg_b.start_mm.x.max(seg_b.end_mm.x);

                                let overlap = (max_x1.min(max_x2) - min_x1.max(min_x2)).max(0.0);
                                if overlap > 1.0 {
                                    let (m_nh, c_pf, noise_mv) =
                                        Self::calculate_crosstalk(overlap, dist, height_mm);
                                    max_crosstalk = max_crosstalk.max(noise_mv);
                                    emi_disturbances.push(EmiDisturbancePair {
                                        aggressor_net: net_a.net_name.clone(),
                                        victim_net: net_b.net_name.clone(),
                                        parallel_coupling_length_mm: overlap,
                                        separation_distance_mm: dist,
                                        mutual_inductance_nh: m_nh,
                                        mutual_capacitance_pf: c_pf,
                                        induced_crosstalk_noise_mv: noise_mv,
                                    });
                                }
                            }
                        } else if is_parallel_v {
                            let dist = (seg_a.start_mm.x - seg_b.start_mm.x).abs();
                            if dist > 0.1 && dist < 12.0 {
                                let min_y1 = seg_a.start_mm.y.min(seg_a.end_mm.y);
                                let max_y1 = seg_a.start_mm.y.max(seg_a.end_mm.y);
                                let min_y2 = seg_b.start_mm.y.min(seg_b.end_mm.y);
                                let max_y2 = seg_b.start_mm.y.max(seg_b.end_mm.y);

                                let overlap = (max_y1.min(max_y2) - min_y1.max(min_y2)).max(0.0);
                                if overlap > 1.0 {
                                    let (m_nh, c_pf, noise_mv) =
                                        Self::calculate_crosstalk(overlap, dist, height_mm);
                                    max_crosstalk = max_crosstalk.max(noise_mv);
                                    emi_disturbances.push(EmiDisturbancePair {
                                        aggressor_net: net_a.net_name.clone(),
                                        victim_net: net_b.net_name.clone(),
                                        parallel_coupling_length_mm: overlap,
                                        separation_distance_mm: dist,
                                        mutual_inductance_nh: m_nh,
                                        mutual_capacitance_pf: c_pf,
                                        induced_crosstalk_noise_mv: noise_mv,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Ground bounce estimation: delta_V = L_gnd * dI/dt (assuming 20mA switching across 1ns)
        let total_l_gnd = net_parasitics
            .iter()
            .filter(|p| p.net_name.contains("GND"))
            .map(|p| p.self_inductance_nh)
            .sum::<f32>()
            .max(1.2);
        let di_dt = 0.020 / 1e-9; // 20mA / 1ns = 2e7 A/s
        let ground_bounce_mv = (total_l_gnd * 1e-9 * di_dt) * 1e3;

        PhysicalWiringRealismReport {
            net_parasitics,
            emi_disturbances,
            max_crosstalk_noise_mv: max_crosstalk,
            estimated_ground_bounce_mv: ground_bounce_mv,
        }
    }

    /// Computes mutual inductance (nH), mutual capacitance (pF), and induced noise voltage (mV).
    fn calculate_crosstalk(
        length_mm: f32,
        dist_mm: f32,
        height_mm: f32,
    ) -> (f32, f32, f32) {
        let length_m = length_mm * 1e-3;
        let dist_m = dist_mm * 1e-3;
        let height_m = height_mm * 1e-3;

        // Mutual inductance: M = (mu_0 * L / 2pi) * ln(1 + 2h / d)
        let m_henry = (VACUUM_PERMEABILITY * length_m / (2.0 * PI)) * (1.0 + 2.0 * height_m / dist_m).ln();
        let m_nh = (m_henry * 1e9).max(0.01);

        // Mutual capacitance: Cm = pi * eps_0 * eps_r * L / ln(pi * d / W + 1)
        let width_m = 0.25e-3;
        let cm_farad = (PI * VACUUM_PERMITTIVITY * FR4_RELATIVE_PERMITTIVITY * length_m)
            / (PI * dist_m / width_m + 1.0).ln();
        let cm_pf = (cm_farad * 1e12).max(0.01);

        // Induced noise voltage: V_noise = M * dI/dt + (Cm / C_load) * delta_V
        // For standard 3.3V CMOS with 1ns transition time:
        let di_dt = 0.015 / 1e-9; // 15mA / 1ns
        let v_ind_l = m_henry * di_dt;
        let v_ind_c = (cm_farad / 5.0e-12) * 3.3; // 5pF victim load
        let total_noise_mv = ((v_ind_l + v_ind_c) * 1000.0).clamp(0.1, 850.0);

        (m_nh, cm_pf, total_noise_mv)
    }
}
