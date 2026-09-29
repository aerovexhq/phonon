#![deny(unsafe_code)]

//! High-Performance Parallel Rayon Benchmark for Quantum Acoustic Cavity Resonators,
//! Surface Acoustic Wave Qubits & Phonon-Mediated Entanglement.

use super::cqa_master_equation::CqaMasterEquationSolver;
use super::phonon_entanglement_solver::{evaluate_saw_beam_splitter, PhononEntanglementSolver};
use phonon_models::quantum_acoustic::{
    BraggAcousticMirror, InterdigitalTransducer, SawBeamSplitter, SawCavity, SawQubitCoupling,
    SawSubstrateMaterial, TransmonQubit, VirtualPhononBus,
};
use rayon::prelude::*;
use std::time::Instant;

/// Individual outcome of a single quantum acoustic parameter sweep.
#[derive(Debug, Clone, Copy)]
pub struct QuantumAcousticSweepResult {
    /// Center frequency in GHz.
    pub center_freq_ghz: f64,
    /// IDT conversion efficiency $\eta_{IDT} \in [0, 1]$.
    pub idt_conversion_efficiency: f64,
    /// Vacuum coupling rate $g / 2\pi$ in MHz.
    pub coupling_rate_mhz: f64,
    /// Resonant Cooperativity $\mathcal{C}_{cqa}$.
    pub cooperativity: f64,
    /// Resonant SWAP gate Fock state $|1\rangle$ generation fidelity $F_{swap} \in [0, 1]$.
    pub swap_fidelity: f64,
    /// Effective virtual exchange interaction $J_{eff} / 2\pi$ in MHz.
    pub j_eff_mhz: f64,
    /// Remote two-qubit Bell state entanglement fidelity $F_{bell} \in [0, 1]$.
    pub bell_fidelity: f64,
    /// Quantum concurrence $\mathcal{C} \in [0, 1]$.
    pub concurrence: f64,
    /// Acoustic beam splitter HOM visibility $V_{HOM} \in [0, 1]$.
    pub hom_visibility: f64,
}

/// Comprehensive Statistical Report of the Quantum Acoustic Benchmark.
#[derive(Debug, Clone, Copy)]
pub struct QuantumAcousticBenchmarkReport {
    /// Total number of sweeps executed.
    pub total_sweeps: usize,
    /// Elapsed wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Processing throughput in sweeps per second.
    pub throughput_sweeps_per_sec: f64,
    /// Mean IDT microwave-to-phonon conversion efficiency.
    pub mean_conversion_efficiency: f64,
    /// Mean vacuum coupling rate $g / 2\pi$ in MHz.
    pub mean_coupling_rate_mhz: f64,
    /// Mean cooperativity.
    pub mean_cooperativity: f64,
    /// Fraction of parameter space satisfying strong coupling ($\mathcal{C} > 1$).
    pub strong_coupling_fraction: f64,
    /// Mean single-phonon Fock state generation fidelity $F_{swap}$.
    pub mean_swap_fidelity: f64,
    /// Minimum single-phonon Fock state generation fidelity.
    pub min_swap_fidelity: f64,
    /// Mean remote Bell state entanglement fidelity $F_{bell}$.
    pub mean_bell_fidelity: f64,
    /// Minimum Bell state entanglement fidelity.
    pub min_bell_fidelity: f64,
    /// Mean quantum concurrence.
    pub mean_concurrence: f64,
    /// Mean acoustic beam splitter HOM visibility.
    pub mean_hom_visibility: f64,
}

/// Runs the parallel Rayon benchmark across `num_sweeps` configurations.
pub fn run_quantum_acoustic_benchmark(num_sweeps: usize) -> QuantumAcousticBenchmarkReport {
    let start_time = Instant::now();

    let results: Vec<QuantumAcousticSweepResult> = (0..num_sweeps)
        .into_par_iter()
        .map(|idx| {
            // Parameter variations:
            // Center frequency around 4.5 GHz ± 0.5 GHz
            let freq_offset = ((idx % 100) as f64 - 50.0) * 1e7; // ± 500 MHz
            let f0 = 4.5e9 + freq_offset;

            // Transducer finger pairs 25 to 45
            let n_pairs = 25 + (idx % 21);
            let aperture = (30.0 + (idx % 20) as f64) * 1e-6; // 30 - 50 um

            let idt = InterdigitalTransducer::new(
                SawSubstrateMaterial::LiNbO3_128YX,
                f0,
                n_pairs,
                aperture,
                50.0,
            );
            let eta = idt.conversion_efficiency(f0);

            // Mirror with 300 to 500 strips
            let n_strips = 300 + (idx % 200);
            let mirror = BraggAcousticMirror::new(n_strips, 0.015, idt.center_wavelength());
            let cavity = SawCavity::new(
                SawSubstrateMaterial::LiNbO3_128YX,
                300e-6,
                aperture,
                idt,
                mirror,
                1.0e5, // Q_int = 100,000
            );

            let v_zpf = cavity.zero_point_voltage(f0);
            let kappa = cavity.phonon_decay_rate(f0);

            // Qubit with f_q = f0 (resonance for SWAP) or detuned for bus
            let t1 = (15.0 + (idx % 15) as f64) * 1e-6; // 15 - 30 us
            let t_phi = (25.0 + (idx % 25) as f64) * 1e-6; // 25 - 50 us
            let qubit1 = TransmonQubit::from_frequency_and_anharmonicity(f0, -250e6, t1, t_phi);

            // Resonant coupling
            let beta = 0.012 + ((idx % 25) as f64) * 0.0004; // 0.012 - 0.022
            let coupling_res = SawQubitCoupling::new(qubit1, f0, kappa, beta, v_zpf);
            let _g_rad = coupling_res.coupling_rate_rad_s();
            let g_mhz = coupling_res.coupling_rate_hz() / 1e6;
            let coop = coupling_res.cooperativity();

            // Analytical SWAP fidelity
            let cqa_solver = CqaMasterEquationSolver::new(coupling_res, 1e4);
            let f_swap = cqa_solver.analytical_swap_fidelity();

            // Virtual phonon bus: detuned by 100 MHz
            let delta_f = 100e6 + ((idx % 50) as f64) * 1e6; // 100 - 150 MHz
            let qubit_bus1 =
                TransmonQubit::from_frequency_and_anharmonicity(f0 + delta_f, -250e6, t1, t_phi);
            let qubit_bus2 =
                TransmonQubit::from_frequency_and_anharmonicity(f0 + delta_f, -250e6, t1, t_phi);

            let coup_bus1 = SawQubitCoupling::new(qubit_bus1, f0, kappa, beta, v_zpf);
            let coup_bus2 = SawQubitCoupling::new(qubit_bus2, f0, kappa, beta, v_zpf);
            let bus = VirtualPhononBus::new(coup_bus1, coup_bus2);
            let j_mhz = bus.effective_exchange_coupling_hz().abs() / 1e6;

            let ent_solver = PhononEntanglementSolver::new(bus);
            let f_bell = ent_solver.analytical_bell_fidelity();
            let concurrence = (2.0 * f_bell - 1.0).clamp(0.0, 1.0);

            // Acoustic beam splitter
            let l_bs = (80.0 + ((idx % 40) as f64)) * 1e-6;
            let c_bs = std::f64::consts::PI / (4.0 * 100e-6); // nominal 100 um 50:50
            let bs = SawBeamSplitter::new(l_bs, c_bs);
            let hom = evaluate_saw_beam_splitter(&bs);

            QuantumAcousticSweepResult {
                center_freq_ghz: f0 / 1e9,
                idt_conversion_efficiency: eta,
                coupling_rate_mhz: g_mhz,
                cooperativity: coop,
                swap_fidelity: f_swap,
                j_eff_mhz: j_mhz,
                bell_fidelity: f_bell,
                concurrence,
                hom_visibility: hom.hom_visibility,
            }
        })
        .collect();

    let elapsed = start_time.elapsed().as_secs_f64();
    let total = results.len();
    let throughput = if elapsed > 0.0 {
        total as f64 / elapsed
    } else {
        0.0
    };

    let mut sum_eta = 0.0;
    let mut sum_g = 0.0;
    let mut sum_coop = 0.0;
    let mut strong_count = 0;
    let mut sum_swap = 0.0;
    let mut min_swap = 1.0f64;
    let mut sum_bell = 0.0;
    let mut min_bell = 1.0f64;
    let mut sum_conc = 0.0;
    let mut sum_hom = 0.0;

    for r in &results {
        sum_eta += r.idt_conversion_efficiency;
        sum_g += r.coupling_rate_mhz;
        sum_coop += r.cooperativity;
        if r.cooperativity > 1.0 {
            strong_count += 1;
        }
        sum_swap += r.swap_fidelity;
        if r.swap_fidelity < min_swap {
            min_swap = r.swap_fidelity;
        }
        sum_bell += r.bell_fidelity;
        if r.bell_fidelity < min_bell {
            min_bell = r.bell_fidelity;
        }
        sum_conc += r.concurrence;
        sum_hom += r.hom_visibility;
    }

    let n = total as f64;
    QuantumAcousticBenchmarkReport {
        total_sweeps: total,
        elapsed_seconds: elapsed,
        throughput_sweeps_per_sec: throughput,
        mean_conversion_efficiency: sum_eta / n,
        mean_coupling_rate_mhz: sum_g / n,
        mean_cooperativity: sum_coop / n,
        strong_coupling_fraction: (strong_count as f64) / n,
        mean_swap_fidelity: sum_swap / n,
        min_swap_fidelity: min_swap,
        mean_bell_fidelity: sum_bell / n,
        min_bell_fidelity: min_bell,
        mean_concurrence: sum_conc / n,
        mean_hom_visibility: sum_hom / n,
    }
}
