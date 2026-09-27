//! Integration tests for Neuromorphic Computing:
//! Memristive Synaptic Crossbars (VMM and parasitic wire IR drop),
//! Spiking Neural Networks (LIF/AdEx action potentials), STDP plasticity,
//! and non-linear MNA KCL conservation.

use phonon_core::CircuitGraph;
use phonon_models::memristor::crossbar::{CrossbarCellType, MemristiveCrossbarModel};
use phonon_models::memristor::neuron::{NeuronState, SpikingNeuronModel};
use phonon_models::memristor::stdp::SpikeTimingPlasticityModel;
use phonon_solver::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::kcl_probe::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_memristive_crossbar_vmm_and_wire_parasitics() {
    let rows = 4;
    let cols = 4;
    let crossbar = MemristiveCrossbarModel::new(rows, cols, CrossbarCellType::Passive1R, 1e-4);

    let v_in = vec![1.0, 0.5, 0.2, 0.0];

    // Ideal Vector-Matrix Multiplication (R_wire = 0)
    let i_ideal = crossbar.evaluate_ideal_vmm(&v_in);
    assert_eq!(i_ideal.len(), cols);

    // Each column should sum: 1e-4 * (1.0 + 0.5 + 0.2 + 0.0) = 1.7e-4 A
    let expected_col_sum = 1e-4 * 1.7;
    for &i_val in &i_ideal {
        assert!(
            (i_val - expected_col_sum).abs() < 1e-10,
            "Ideal VMM mismatch: got {}, expected {}",
            i_val,
            expected_col_sum
        );
    }

    // Parasitic wire resistance computation
    let (i_parasitic, total_power) = crossbar.evaluate_parasitic_vmm(&v_in);
    let rmse = crossbar.vmm_error_rmse(&v_in);

    assert!(total_power > 0.0, "VMM dissipation must be positive");

    // Wire resistance reduces total delivered current (IR drop along lines)
    for c in 0..cols {
        assert!(
            i_parasitic[c] <= i_ideal[c],
            "Parasitic resistance must attenuate column output current: parasitic = {}, ideal = {}",
            i_parasitic[c],
            i_ideal[c]
        );
    }
    assert!(
        rmse > 0.0 && rmse < expected_col_sum * 0.1,
        "RMSE should reflect modest IR drop degradation (<10%): rmse = {}",
        rmse
    );
}

#[test]
fn test_spiking_neuron_action_potential_emission_and_refractory() {
    let neuron = SpikingNeuronModel::neuromorphic_cmos();
    let mut state = NeuronState::new(neuron.v_rest_volts);

    let dt = 1e-5; // 10 microseconds
    let i_stim = 150e-9; // 150 nA injection current (> 80 nA rheobase threshold)
    let n_steps = 1000; // 10 ms total simulation

    let mut spike_count = 0;
    let mut fired_times = Vec::new();

    for step in 0..n_steps {
        let t = (step as f64) * dt;
        let (next_state, spike) = neuron.step(state, i_stim, 1, t, dt);
        state = next_state;
        if spike.is_some() {
            spike_count += 1;
            fired_times.push(t);
        }
    }

    assert!(
        spike_count >= 2,
        "Neuron must fire at least 2 spikes under 150 nA sustained current: got {}",
        spike_count
    );

    // Verify inter-spike interval is strictly greater than refractory period
    if fired_times.len() >= 2 {
        let isi = fired_times[1] - fired_times[0];
        assert!(
            isi >= neuron.t_refractory_s,
            "Inter-spike interval ({} s) must exceed refractory period ({} s)",
            isi,
            neuron.t_refractory_s
        );
    }
}

#[test]
fn test_spike_timing_dependent_plasticity_stdp() {
    let stdp = SpikeTimingPlasticityModel::cortical_stdp(1e-6, 1e-4);

    // 1. Long-Term Potentiation (LTP): Pre-synaptic spike occurs BEFORE post-synaptic (delta_t > 0)
    let delta_t_pre_before_post = 0.010; // +10 ms
    let dw_ltp = stdp.delta_weight_raw(delta_t_pre_before_post);
    assert!(
        dw_ltp > 0.0,
        "LTP must increase synaptic weight when pre fires before post: got {}",
        dw_ltp
    );

    // 2. Long-Term Depression (LTD): Post-synaptic spike occurs BEFORE pre-synaptic (delta_t < 0)
    let delta_t_post_before_pre = -0.010; // -10 ms
    let dw_ltd = stdp.delta_weight_raw(delta_t_post_before_pre);
    assert!(
        dw_ltd < 0.0,
        "LTD must decrease synaptic weight when post fires before pre: got {}",
        dw_ltd
    );

    // 3. Weight boundary saturation
    let w_mid = 5e-5;
    let w_near_max = 9.9e-5;
    let w_updated_mid = stdp.update_weight(w_mid, delta_t_pre_before_post);
    let w_updated_max = stdp.update_weight(w_near_max, delta_t_pre_before_post);
    let delta_mid = w_updated_mid - w_mid;
    let delta_near_max = w_updated_max - w_near_max;

    assert!(
        delta_near_max < delta_mid,
        "Soft saturation must dampen weight increase near upper bound: delta_near_max = {}, delta_mid = {}",
        delta_near_max,
        delta_mid
    );
}

#[test]
fn test_snn_neuron_circuit_dc_mna_and_kcl() {
    let mut graph = CircuitGraph::new();

    // Input branch: V_STIM (node 1) -> R_IN (node 1 to node 2)
    // SpikingNeuron: input at node 2, output at node 3
    // Output branch: R_LOAD (node 3 to ground)
    graph.add_voltage_source("V_STIM", "1", "0", 1.5).unwrap();
    graph.add_resistor("R_IN", "1", "2", 1e4).unwrap();
    graph
        .add_spiking_neuron("NEURON1", "2", "3", 0.5, 0.0)
        .unwrap();
    graph.add_resistor("R_LOAD", "3", "0", 1e3).unwrap();

    let neuron_model = SpikingNeuronModel::neuromorphic_cmos();
    let mut context = ModelContext::new();
    context.set_neuron_model("NEURON1", neuron_model);

    let opts = NewtonOptions::default();
    let solution = solve_dc_non_linear(&graph, &context, &opts)
        .expect("Non-linear DC solver must converge for SpikingNeuron circuit");

    let v2 = solution.node_voltages[graph.get_node("2").unwrap().index()];
    let v3 = solution.node_voltages[graph.get_node("3").unwrap().index()];

    // Input node 2 is divided by R_IN and neuron membrane resistance
    assert!(
        v2 > 0.0,
        "Membrane potential v2 must be positive: got {}",
        v2
    );

    // Since v2 is high (> 0.5 V), the output should fire / pull to high
    assert!(v3 > 0.0, "Neuron output v3 must be non-zero: got {}", v3);

    // Verify KCL
    let empty_cap = HashMap::new();
    let kcl_report = verify_kcl_dynamic(
        &graph,
        &solution.node_voltages,
        &solution.branch_currents,
        &empty_cap,
        Some(&context),
        1e-3,
        1e-6,
    );

    assert!(
        kcl_report.is_valid,
        "KCL violated in SpikingNeuron circuit: max residual = {} A at node {:?}",
        kcl_report.max_residual, kcl_report.worst_node
    );
}
