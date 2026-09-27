//! Property-based validation asserting Kirchhoff's Current Law (KCL) conservation across hundreds of randomized circuit networks.

use phonon_core::CircuitGraph;
use phonon_solver::{solve_dc_linear, SolverOptions};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_random_connected_resistor_network_kcl(
        num_nodes in 3usize..15,
        v_source in 1.0f64..24.0,
        extra_edges in 1usize..20,
    ) {
        let mut graph = CircuitGraph::new();

        // Node 0 is Ground
        // Connect a primary DC voltage source from node_1 to Ground
        graph.add_voltage_source("V_main", "node_1", "0", v_source).unwrap();

        // Build a guaranteed connected spanning tree from node_1 to all nodes
        for i in 2..=num_nodes {
            let parent_idx = (i - 1).max(1);
            let from = format!("node_{}", parent_idx);
            let to = format!("node_{}", i);
            let r_val = 100.0 + (i as f64) * 50.0;
            graph.add_resistor(&format!("R_tree_{}", i), &from, &to, r_val).unwrap();
        }

        // Add additional cross-connecting resistors
        for k in 0..extra_edges {
            let n1 = 1 + (k % num_nodes);
            let n2 = 1 + ((k * 3 + 1) % num_nodes);
            if n1 != n2 {
                let from = format!("node_{}", n1);
                let to = format!("node_{}", n2);
                let r_val = 500.0 + (k as f64) * 100.0;
                let _ = graph.add_resistor(&format!("R_extra_{}", k), &from, &to, r_val);
            }
        }

        // Solve DC operating point
        let sol = solve_dc_linear(&graph, &SolverOptions::default())
            .expect("Connected linear resistor network must always converge");

        // Validate KCL at every active node (except node_1 which has the voltage source branch current)
        for i in 2..=num_nodes {
            let node_name = format!("node_{}", i);
            let node_id = graph.get_node(&node_name).unwrap();
            let v_i = sol.node_voltage(node_id);

            let mut kcl_sum = 0.0;
            for comp in graph.components() {
                if let phonon_core::ComponentRecord::Resistor { pos, neg, resistance, .. } = comp {
                    if *pos == node_id {
                        let v_nbr = sol.node_voltage(*neg);
                        kcl_sum += (v_i - v_nbr) / resistance;
                    } else if *neg == node_id {
                        let v_nbr = sol.node_voltage(*pos);
                        kcl_sum += (v_i - v_nbr) / resistance;
                    }
                }
            }

            prop_assert!(
                kcl_sum.abs() < 1e-9,
                "KCL violated at node {}: sum = {:e} A (V = {} V)",
                node_name,
                kcl_sum,
                v_i
            );
        }
    }
}
