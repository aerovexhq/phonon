//! Integration test: Coupled microstrip transmission lines and high-speed digital crosstalk dynamics (NEXT & FEXT).
//! Validates:
//! 1. Modal impedance decomposition ($Z_{0e}, Z_{0o}$), characteristic impedance $Z_0 = \sqrt{Z_{0e} Z_{0o}}$, and NEXT coupling coefficient $K_B$.
//! 2. Far-End Crosstalk (FEXT) modal delay dispersion coefficient $K_F = (\tau_e - \tau_o)/(2 \ell)$.
//! 3. Dynamic transient pulse transmission through synthesized coupled line pair with victim line induced crosstalk.

use phonon_core::CircuitGraph;
use phonon_models::tline::coupled_microstrip::CoupledMicrostripLine;
use phonon_solver::mna::{ModelContext, NewtonOptions};
use phonon_solver::solve_transient;
use phonon_solver::transient::{
    IntegrationMethod, StepControlOptions, TimeWaveform, TransientOptions,
};

#[test]
fn test_coupled_microstrip_modal_parameters_and_crosstalk_coefficients() {
    let z0e = 60.0;
    let z0o = 40.0;
    let tau_e = 1.1e-9;
    let tau_o = 0.9e-9;
    let len_m = 0.15;

    let line = CoupledMicrostripLine::new(z0e, z0o, tau_e, tau_o, len_m);

    // 1. Single-ended impedance: sqrt(60 * 40) = sqrt(2400) = 48.98979 Ohm
    let z0 = line.single_ended_impedance();
    assert!((z0 - (2400.0f64).sqrt()).abs() < 1e-9);

    // 2. Backward / Near-End Crosstalk (NEXT) coefficient: (60 - 40)/(60 + 40) = 0.20
    let kb = line.next_coefficient();
    assert!((kb - 0.20).abs() < 1e-12);

    // 3. Peak NEXT voltage for a 3.3V LVCMOS step: 0.20 * 3.3V = 0.66 V
    let v_step = 3.3;
    let v_next = line.peak_next_voltage(v_step);
    assert!((v_next - 0.66).abs() < 1e-12);

    // 4. Far-End Crosstalk (FEXT) coefficient: (tau_e - tau_o) / (2 * len) = 0.2e-9 / 0.30 = 0.66667 ns/m
    let kf = line.fext_coefficient();
    let expected_kf = (1.1e-9 - 0.9e-9) / (2.0 * len_m);
    assert!((kf - expected_kf).abs() < 1e-15);

    // 5. Peak FEXT voltage for 3.3V step with 100 ps rise time:
    // V_FEXT = -0.5 * (1.1 - 0.9)e-9 * (3.3 / 100e-12) = -0.5 * 0.2 * 33 = -3.3 V
    let v_fext = line.peak_fext_voltage(3.3, 100.0e-12);
    assert!((v_fext - (-3.3)).abs() < 1e-9);
}

#[test]
fn test_coupled_line_transient_pulse_and_crosstalk() {
    let z0e = 58.0;
    let z0o = 43.0;
    let tau_e = 1.05e-9;
    let tau_o = 0.95e-9;
    let len_m = 0.10;

    let line = CoupledMicrostripLine::new(z0e, z0o, tau_e, tau_o, len_m);
    let z0 = line.single_ended_impedance(); // approx 50 Ohm

    let mut graph = CircuitGraph::new();

    // Synthesize coupled pair: aggressor ("a_in", "a_out") and victim ("v_in", "v_out")
    line.synthesize_subcircuit(&mut graph, "CP", "a_in", "a_out", "v_in", "v_out", "0")
        .expect("Subcircuit synthesis failed");

    // Aggressor drive: 2.0V step source with Rs = z0
    graph.add_voltage_source("V_SRC", "src", "0", 0.0).unwrap();
    graph.add_resistor("R_SRC", "src", "a_in", z0).unwrap();
    // Aggressor termination: matched load RL = z0
    graph.add_resistor("RL_AGG", "a_out", "0", z0).unwrap();

    // Victim line terminations: matched resistors at both ends (near and far)
    graph.add_resistor("RL_VIC_NEAR", "v_in", "0", z0).unwrap();
    graph.add_resistor("RL_VIC_FAR", "v_out", "0", z0).unwrap();

    let n_a_in = graph.get_node("a_in").unwrap();
    let n_a_out = graph.get_node("a_out").unwrap();
    let n_v_in = graph.get_node("v_in").unwrap();
    let n_v_out = graph.get_node("v_out").unwrap();

    let mut waveforms = std::collections::HashMap::new();
    waveforms.insert(
        "V_SRC".to_string(),
        TimeWaveform::Pwl {
            points: vec![(0.0, 0.0), (0.1e-9, 2.0), (4.0e-9, 2.0)],
        },
    );

    let options = TransientOptions {
        tstop: 3.0e-9,
        tstep: 0.01e-9,
        method: IntegrationMethod::Trapezoidal,
        waveforms,
        step_control: StepControlOptions {
            min_step: 1e-12,
            max_step: 0.01e-9,
            reltol: 1e-3,
            abstol: 1e-6,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        ..Default::default()
    };

    let solution = solve_transient(&graph, &ModelContext::new(), &options)
        .expect("Transient simulation failed");

    let a_in_wave = solution.node_waveform(n_a_in);
    let a_out_wave = solution.node_waveform(n_a_out);
    let v_in_wave = solution.node_waveform(n_v_in);
    let v_out_wave = solution.node_waveform(n_v_out);

    // 1. Aggressor input steps to ~1.0 V (half of 2.0V source) after capacitive displacement settles
    // and before round-trip far-end reflection arrives (at 2*tau = 2.0 ns)
    for &(t, v) in &a_in_wave {
        if (0.4e-9..=1.9e-9).contains(&t) {
            assert!(
                (v - 1.0).abs() < 0.05,
                "Aggressor input voltage expected ~1.0V, got {v}V at t={t}"
            );
        }
    }

    // 2. Aggressor output arrives around tau_avg = 1.0 ns
    for &(t, v) in &a_out_wave {
        if t < 0.9e-9 {
            assert!(
                v.abs() < 0.02,
                "Aggressor output showed early signal at t={t}: {v}V"
            );
        }
        if t >= 1.4e-9 {
            assert!(
                (v - 1.0).abs() < 0.05,
                "Aggressor output expected ~1.0V at t={t}, got {v}V"
            );
        }
    }

    // 3. Victim near end (NEXT): induced crosstalk spike during the rise time (0 to 0.2 ns)
    let max_v_next = v_in_wave
        .iter()
        .filter(|(t, _)| *t <= 0.3e-9)
        .map(|(_, v)| v.abs())
        .fold(0.0f64, f64::max);
    assert!(
        max_v_next > 0.01,
        "Expected measurable NEXT voltage on victim near end, got max {max_v_next}V"
    );

    // 4. Victim far end (FEXT): induced crosstalk spike when pulse arrives at far end (0.95 to 1.15 ns)
    let max_v_fext = v_out_wave
        .iter()
        .filter(|(t, _)| (0.9e-9..=1.2e-9).contains(t))
        .map(|(_, v)| v.abs())
        .fold(0.0f64, f64::max);
    assert!(
        max_v_fext > 0.01,
        "Expected measurable FEXT voltage on victim far end, got max {max_v_fext}V"
    );
}
