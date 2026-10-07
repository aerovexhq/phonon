#![deny(unsafe_code)]

//! Phonon Solver: high-performance sparse linear algebra, Modified Nodal Analysis (MNA),
//! Markowitz threshold pivoting, dynamic TR-BDF2 transient solver, and physical conservation probes.

pub mod acoustic;
pub mod acoustic_chern_circulator;
pub mod acoustic_skyrmion_router;
pub mod acoustic_domain_wall_soliton;
pub mod valley_acoustic_multiplexer;
pub mod quadrupole_shg;
pub mod quadrupole_parametric;
pub mod corner_harmonic_doubler;
pub mod synthetic_4d_qhe;
pub mod pt_symmetric_acoustic;
pub mod acoustic_bic;
pub mod euler_acoustic;
pub mod octupole_insulator;
pub mod aah_quasicrystal;
pub mod valley_hall_vortex;
pub mod skyrmion_deflector;
pub mod floquet_frequency_dimension;
pub mod non_hermitian_corner_laser;
pub mod directional_radiation;
pub mod atmospheric_neutron;
pub mod thermal_vacuum;
pub mod space_avionics_bus;
pub mod rhbd_self_healing;
pub mod production_economics;
pub mod chiplet_packaging;
pub mod electrothermal_throttling;
pub mod pdn_droop;
pub mod silicon_aging;
pub mod wafer_yield;
pub mod dse_optimization;
pub mod silicon_lifecycle;
pub mod wavepacket_scattering;
pub mod skyrmion_reservoir;
pub mod acoustic_holonomic_processor;
pub mod acoustic_metasurface_holography;
pub mod acoustic_metasurface_hologram;
pub mod acoustic_microcomb_soliton;
pub mod acoustically_levitated_nanoparticle;
pub mod acoustoelectric;
pub mod acoustoelectric_moire;
pub mod acoustomagnonic_comb;
pub mod acoustomagnonic_haloscope;
pub mod acoustomagnonic_polariton_laser;
pub mod afm_spintronics;
pub mod assets;
pub mod audio_dsp_synth;
pub mod axion_electrodynamics;
pub mod braiding_switchyard;
pub mod cavity_acoustodynamical_spin;
pub mod cavity_acoustomagnonic;
pub mod cavity_magnomechanics;
pub mod cavity_magnon_polariton_comb;
pub mod optomagnonic_comb;
pub mod cavity_spintronics;
pub mod cluster;
pub mod chiral_acoustic_router;
pub mod chiral_axion_circulator;
pub mod chiral_chern_anyon_braiding;
pub mod chiral_floquet_hall_transistor;
pub mod chiral_frequency_bin_bell_analyzer;
pub mod chiral_hinge_axion_soliton;
pub mod chiral_holographic_beamforming;
pub mod chiral_moire_fractional_chern;
pub mod chiral_phonon;
pub mod chiral_phonon_sc;
pub mod chiral_phonon_spin_mechanics;
pub mod chiral_phonon_magnon_isolator;
pub mod chiral_polariton;
pub mod chiral_polariton_circulator;
pub mod josephson_parametric_amplifier;
pub mod chiral_quantum_hall_pfaffian;
pub mod chiral_spin_seebeck;
pub mod chiral_spintronic_memristor;
pub mod cqed;
pub mod diamond_nv;
pub mod em;
pub mod ep_sensor;
pub mod error;
pub mod floquet;
pub mod floquet_acoustic_chern;
pub mod floquet_anyon_braiding;
pub mod floquet_corner_transduction;
pub mod floquet_majorana_braiding_processor;
pub mod floquet_topological;
pub mod floquet_metasurface;
pub mod floquet_time_crystal;
pub mod floquet_time_crystal_sensor;
pub mod fqh;
pub mod fqh_acoustic_interferometer;
pub mod fqh_interferometer;
pub mod fqh_braiding;
pub mod fractional_chern;
pub mod fractional_hall_parafermion;
pub mod fractional_josephson_parafermion;
pub mod fractional_chern_simons_viscometer;
pub mod moire_skyrmion_anyon_braiding;
pub mod hetero;
pub mod hexagonal_majorana;
pub mod high_harmonic_bloch;
pub mod holonomic_quantum_processor;
pub mod hotp_quadrupole_octupole_metasurface;
pub mod hotp_axion_hinge_circulator;
pub mod fibonacci_anyon_quantum_memory;
pub mod non_hermitian_skin_octupole_laser;
pub mod fractional_qh_entanglement_swapper;
pub mod parafermionic_josephson_interferometer;
pub mod interfacial_superconductivity;
pub mod josephson_vortex_ratchet;
pub mod jtwpa;
pub mod kitwpa;
pub mod kitaev_spin_liquid_braiding;
pub mod kerr_microcomb;
pub mod lidar;
pub mod magnon_bec;
pub mod majorana_chiral_phonon;
pub mod majorana_kramers_network;
pub mod fracton_quadrupole_router;
pub mod disclination_holonomic_processor;
pub mod skyrmion_vortex_polariton;
pub mod twist_defect_lattice;
pub mod pfaffian_quantum_resonator;
pub mod axion_string_memristor;
pub mod skyrmion_anyonic_repeater;
pub mod surface_code_decoder;
pub mod surface_code_transceiver;
pub mod hyperbolic_crystallizer;
pub mod majorana_transmon_hybrid;
pub mod anyonic_neural_synapse;
pub mod chern_heat_engine;
pub mod optomechanical_switchyard;
pub mod saw_soliton_routing;
pub mod spin_optomechanical_bridge;
pub mod braiding_circuit_compiler;
pub mod visual_studio_engine;
pub mod collaboration_fabric;
pub mod gpu_tensor_mesh;
pub mod distributed_mesh;
pub mod neural_circuit_copilot;
pub mod holographic_telemetry;
pub mod generative_diffusion;
pub mod mask_tapeout;
pub mod cryo_testbed;
pub mod quantum_digital_twin;
pub mod cloud_deployment;
pub mod quantum_transceiver;
pub mod molecular_spintronics;
pub mod superradiance_laser;
pub mod topological_axion;
pub mod exceptional_surface;
pub mod soti_corner_resonator;
pub mod lieb_lattice;
pub mod axion_insulator;
pub mod floquet_anyon;
pub mod skyrmionic_memory;
pub mod quadrupole_qubit;
pub mod spin_valley;
pub mod holonomic_quantum;
pub mod acoustomagnonic_squeezing;
pub mod tripartite_router;
pub mod acoustoelectric_transistor;
pub mod teleportation_network;
pub mod valley_heat_pump;
pub mod majorana_braiding_processor;
pub mod floquet_majorana_engine;
pub mod monopole_harmonic_teleporter;
pub mod skyrmion_neural_processor;
pub mod anyonic_knot_coprocessor;
pub mod quasicrystal_phason_router;
pub mod spin_phonon_braiding;
pub mod anyon_condensation;
pub mod corner_state_memory;
pub mod axion_polariton_soliton;
pub mod majorana_surface_memory;
pub mod majorana_surface_code;
pub mod quantum_optomechanical_transducer;
pub mod corner_polariton_microcomb;
pub mod giant_atom_qed;
pub mod metamaterial_circulator_cloak;
pub mod mixed_signal;
pub mod mna;
pub mod moire;
pub mod monte_carlo;
pub mod molecular;
pub mod mvl;
pub mod net;
pub mod neuromorphic;
pub mod non_hermitian;
pub mod non_hermitian_acoustic_laser;
pub mod non_hermitian_chiral_hoti;
pub mod non_hermitian_ep_gyroscope;
pub mod non_hermitian_pt_symmetry;
pub mod non_hermitian_skin;
pub mod non_hermitian_sensor;
pub mod non_hermitian_topo;
pub mod non_reciprocal_phonon_amplifier;
pub mod optics;
pub mod optimization;
pub mod optomechanics;
pub mod opto_acoustic_quantum_repeater;
pub mod opto_electro_phononic_translator;
pub mod phonon_magnon_polariton;
pub mod parallel;
pub mod phononic;
pub mod phononic_anyon_collider;
pub mod phononic_microcomb;
pub mod phononic_neural_annealer;
pub mod phononic_superconducting_majorana;
pub mod phononic_topological;
pub mod plasma;
pub mod polariton_condensate;
pub mod polariton_exceptional_point;
pub mod polariton_quantum_memory;
pub mod programmable_chiral_graph;
pub mod quantum;
pub mod quantum_acoustic;
pub mod quantum_acoustic_anyons;
pub mod quantum_acoustic_waveguide;
pub mod quantum_phonon_teleportation;
pub mod quantum_plasmonics;
pub mod quantum_time_crystal;
pub mod quantum_topological_squeezing;
pub mod quantum_cavity_acoustomechanics;
pub mod quantum_teleportation_waveguide;
pub mod quantum_acoustic_tensor_distillation;
pub mod quantum_acoustic_spin_liquid;
pub mod quantum_dot_spin_shuttle;
pub mod flux_qubit_coupler;
pub mod acoustic_frequency_synthesizer;
pub mod magnon_phonon_repeater;
pub mod levitated_diamond_magnetometer;
pub mod skyrmion_synaptic_router;
pub mod topological_polariton_synapse;
pub mod acoustic_snspd_detector;
pub mod superconducting_quatrit;
pub mod ultracold_fermi_gas_sensor;
pub mod anyon_fusion_synthesizer;
pub mod bec_soliton_interferometer;
pub mod axion_magnon_memory;
pub mod superconducting_anyon_interferometer;
pub mod levitated_superconducting_qubit;
pub mod spin_orbit_majorana_qubit;
pub mod anyon_braiding_processor;
pub mod levitated_qubit_teleporter;
pub mod parafermion_braiding_router;
pub mod levitated_qubit_network;
pub mod spin_valley_polariton;
pub mod skyrmion_majorana_crossbar;
pub mod parafermion_surface_code;
pub mod axion_polariton_transceiver;
pub mod superconducting_ququint;
pub mod topological_valley_hall_router;
pub mod phonon_magnon_polariton_comb;
pub mod quantum_metamaterial_transceiver;
pub mod levitated_nanodiamond_spin_sensor;
pub mod majorana_parafermion_hybrid;
pub mod quantum_metamaterial_beamformer;
pub mod axion_polariton_beam_splitter;
pub mod floquet_chern_isolator;
pub mod valley_chiral_polariton_splitter;
pub mod axion_magnon_polariton_isolator;
pub mod axion_polariton_photonic_isolator;
pub mod superconducting_quoctit;
pub mod axion_polariton_circulator;
pub mod quantum_metamaterial_polariton_laser;
pub mod floquet_chern_parafermion_router;
pub mod fractional_chern_anyon_synthesizer;
pub mod skyrmion_polariton_transceiver;
pub mod majorana_parafermion_lattice;
pub mod floquet_chern_photonic_isolator;
pub mod superconducting_quoctit_crossbar;
pub mod floquet_chern_parafermion_transceiver;
pub mod quantum_metamaterial_multiplexer;
pub mod superconducting_quoctit_processor;
pub mod fqh_pfaffian_router;
pub mod floquet_parafermion_laser;
pub mod skyrmion_majorana_transceiver;
pub mod floquet_parafermion_comb;
pub mod floquet_parafermion_memory;
pub mod skyrmion_parafermion_transceiver;
pub mod fqh_moore_read_processor;
pub mod skyrmion_majorana_memory;
pub mod fqh_moore_read_crossbar;
pub mod skyrmion_parafermion_memory;
pub mod skyrmion_parafermion_processor;
pub mod fqh_moore_read_transceiver;
pub mod floquet_parafermion_repeater;
pub mod fqh_moore_read_memory;
pub mod floquet_parafermion_crossbar;
pub mod floquet_parafermion_processor;
pub mod moore_read_logic_crossbar;
pub mod chiral_skyrmion_magnon_polaron;
pub mod floquet_exceptional_ring_sensor;
pub mod port_hamiltonian;
pub mod symplectic_multirate;
pub mod relay;
pub mod rf;
pub mod sensors;
pub mod skyrmion_braiding_memory;
pub mod skyrmion_phonon_drag;
pub mod snspd;
pub mod space;
pub mod sparse;
pub mod spintronics;
pub mod stno;
pub mod superconducting;
pub mod superconducting_spintronics;
pub mod synthesis;
pub mod topological;
pub mod topological_acoustic_axion;
pub mod topological_soliton_comb;
pub mod topological_weyl_acoustics;
pub mod topological_majorana_braiding;
pub mod topological_chern_circulator;
pub mod topological_corner_laser;
pub mod topological_moire_polariton;
pub mod non_hermitian_skin_amplifier;
pub mod non_hermitian_edge_soliton;
pub mod phonon_exciton_polariton;
pub mod floquet_synthetic_gauge;
pub mod topological_time_crystal;
pub mod topological_acoustic_skyrmion;
pub mod topological_acoustic_fracton;
pub mod twisted_bilayer_moire_polariton;
pub mod twisted_bilayer_topological_superfluid;
pub mod tensor_gauge_monopole_sensor;
pub mod quantum_acoustic_surface_code;
pub mod non_hermitian_quadrupole_laser;
pub mod sensitivity;
pub mod transient;
pub mod valley_acoustic;
pub mod valleytronics;
pub mod verification;
pub mod wakefield;
pub mod polariton_waveguide;
pub mod weyl_semimetal;
pub mod jtwpa_simulator;
pub mod non_abelian_holonomic;
pub mod twisted_moire_superlattice;
pub mod protected_braiding_lattice;
pub mod optomechanical_squeezing;
pub mod universal_braiding_processor;
pub mod chiral_edge_magnetoplasmon;
pub mod polariton_bec_vortices;
pub mod synthetic_dimension_router;
pub mod non_hermitian_skin_laser;
pub mod moire_polariton_comb;
pub mod holonomic_braiding_coprocessor;
pub mod valley_chiral_isolator;
pub mod floquet_spinhall_circulator;
pub mod octupole_dislocation_router;
pub mod chiral_majorana_braiding;

pub use acoustic::{
    AcousticBenchmarkReport, AcousticBenchmarkRunner, AcousticLinkSimulator, AcousticRealismTier,
    AcousticRoom, AcousticStepResult, FdtdResult,
};
pub use acoustic_metasurface_holography::*;
pub use acoustic_microcomb_soliton::*;
pub use acoustoelectric::*;
pub use acoustoelectric_moire::*;
pub use afm_spintronics::*;
pub use assets::{
    AcousticRayHit, AssetBenchmarkReport, AssetBenchmarkRunner, MeshInstance, MultiPhysicsBvh,
    MultiPhysicsBvhNode, MultiPhysicsScene, OpticalRayHit, RfTransmissionResult, WorldTriangle,
};
pub use axion_electrodynamics::*;
pub use cavity_acoustomagnonic::*;
pub use cavity_magnomechanics::*;
pub use cavity_magnon_polariton_comb::*;
pub use cavity_spintronics::*;
pub use chiral_phonon::*;
pub use chiral_phonon_sc::*;
pub use chiral_phonon_spin_mechanics::*;
pub use chiral_polariton::*;
pub use chiral_spin_seebeck::*;
pub use chiral_spintronic_memristor::*;
pub use cqed::{
    CqedBenchmarkReport, CqedBenchmarkRunner, DispersiveReadoutResult, DispersiveReadoutSolver,
    TransmonSpectrumSolution, TransmonSpectrumSolver,
};
pub use diamond_nv::*;
pub use em::{
    AntennaElectrodynamicSolver, EmBenchmarkReport, EmBenchmarkRunner, EmLinkResult,
    EmPropagationScene, EmWaveSolver, EmitterBenchmarkReport, EmitterBenchmarkRunner,
    GroundStation, LinkEvaluationParams, ObservationPoint, PacketTransmissionResult,
    ProtocolBenchmarkReport, ProtocolBenchmarkRunner, RadiationSphereResult, RfRealismTier,
    RfTierEngine, RfTransceiverSolver, SatelliteNode, SpaceLinkBudgetResult, SpaceLinkSolver,
    TierChannelResult, TransceiverLinkResult, TransientBurstResult, WifiLinkSimulator,
};
pub use error::SolverError;
pub use floquet::*;
pub use floquet_acoustic_chern::*;
pub use floquet_anyon_braiding::*;
pub use floquet_corner_transduction::*;
pub use floquet_topological::*;
pub use fqh::*;
pub use fqh_acoustic_interferometer::*;
pub use fractional_chern::*;
pub use hetero::{
    HeteroCpuBenchmarkResult, HeteroCpuBenchmarkRunner, HeteroCpuOptimizer,
    HeteroOptimizationCandidate, PipelineTimingReport, TimingPathAnalyzer,
};
pub use hexagonal_majorana::*;
pub use high_harmonic_bloch::*;
pub use interfacial_superconductivity::*;
pub use josephson_vortex_ratchet::*;
pub use jtwpa::{
    CoupledModeResult, CoupledModeSolver, JtwpaBenchmarkReport, JtwpaBenchmarkRunner,
    QuantumNoiseResult, QuantumNoiseSolver,
};
pub use kitwpa::*;
pub use lidar::{
    Aabb, BvhHit, BvhNode, BvhPrimitive, BvhTree, LidarBenchmarkReport, LidarBenchmarkRunner,
    LidarPoint, LidarPointCloud, TofLidarEngine,
};
pub use magnon_bec::*;
pub use majorana_chiral_phonon::*;
pub use metamaterial_circulator_cloak::*;
pub use mixed_signal::{
    parse_verilog_module, solve_mixed_signal, BoundaryAdc, BoundaryDac, DacSmoothing,
    DigitalEngine, DigitalEvent, DigitalLogicGate, DigitalTraceStep, GateKind, HdlParseError,
    LogicState, MixedSignalCircuit, MixedSignalOptions, MixedSignalSolution, MixedSignalStepReport,
    MixedSignalSynchronizer, ParsedHdlModule,
};
pub use mna::{
    assemble_mna_dc, solve_dc_linear, solve_dc_non_linear, DcSolution, MnaSystem, ModelContext,
    NewtonOptions, SolverOptions,
};
pub use moire::*;
pub use molecular::{
    Cmos3nmBaseline, MolecularBenchmarkRunner, MolecularCandidate, MolecularComparisonReport,
    MolecularLogicSynthesizer, SelfConsistentNegfConfig, SelfConsistentNegfResult,
    SelfConsistentNegfSolver, SynthesizedMolecularLogic, TargetLogicFunction,
};
pub use mvl::{
    MvlBenchmarkReport, MvlBenchmarkRunner, TernaryAdderEngine, TernaryCircuitSolver,
    TernarySolverError,
};
pub use net::{
    CoSimStepReport, NetworkBenchmarkReport, NetworkBenchmarkRunner, NetworkCoSimulator,
};
pub use neuromorphic::{
    generate_lorenz63, generate_mackey_glass, generate_narma10, LifNeuron, LiquidStateMachine,
    LsmConfig, MemristorState, NeuromorphicBenchmarkReport, NeuromorphicBenchmarkRunner,
    ReservoirSolver, SnnTrajectory, SpikingCrossbarNetwork, StdpParams, TrainedReadout,
    WindowFunction,
};
pub use non_hermitian::{
    LaserSimulationResult, MaxwellBlochSolver, NonHermitianEigenResult, NonHermitianEigensolver,
    TopologicalLaserBenchmarkReport, TopologicalLaserBenchmarkRunner,
};
pub use non_hermitian_chiral_hoti::*;
pub use non_hermitian_ep_gyroscope::*;
pub use non_hermitian_pt_symmetry::*;
pub use non_hermitian_skin::*;
pub use non_hermitian_topo::*;
pub use non_hermitian_acoustic_laser::*;
pub use non_reciprocal_phonon_amplifier::*;
pub use optics::{
    AabbBox, CheckerPlane, LightSource, OffscreenPerceptionEngine, OpticalBenchmarkReport,
    OpticalBenchmarkRunner, OpticalRealismTier, PerceptionFrame, RayHit, SceneObject, Sphere,
    SyntheticScene,
};
pub use optimization::{EngineConfig, InverseDesignEngine};
pub use optomechanics::{
    CavityOptomechanicsBenchmarkReport, CavityOptomechanicsBenchmarkRunner, CovarianceResult,
    LangevinSdeSolver, OptomechanicalMasterEquationSolver, TrajectoryResult,
};
pub use parallel::{
    solve_torn_dc, CircuitSoA, DiakopticsSolver, DiodeSoA, MosfetSoA, PartitionedCircuit,
    ResistorSoA, SubcircuitLocalSolution,
};
pub use phononic::{
    AcousticGateVerificationResult, AcousticLogicSolver, AcousticWaveformTrace, ContinuumNode,
    ContinuumSolver2D, PhononicBenchmarkReport, PhononicBenchmarkRunner,
};
pub use phononic_microcomb::*;
pub use phononic_neural_annealer::*;
pub use phononic_topological::*;
pub use plasma::{
    AlfvenFluxNode, AlfvenMhdConfig, AlfvenMhdStepper, BorisPicTracker, GradShafranovGrid,
    GradShafranovSolution, GradShafranovSolver, OrbitTopology, ParticleOrbitReport,
    TokamakBenchmarkReport, TokamakBenchmarkRunner, TokamakScenario,
};
pub use polariton_condensate::*;
pub use polariton_exceptional_point::*;
pub use quantum::{
    CovarianceMatrix4x4, OptomechanicalBenchmarkReport, OptomechanicalBenchmarkRunner,
    OptomechanicalQleSolver, QuantumTransductionMetrics, QuantumTransductionSolver,
    TransducerTechnology, TransductionEvaluationPoint,
};
pub use quantum_acoustic::{
    evaluate_saw_beam_splitter, run_quantum_acoustic_benchmark, CqaMasterEquationSolver,
    HomRoutingReport, PhononEntanglementSolver, QuantumAcousticBenchmarkReport,
    QuantumAcousticSweepResult, QubitPhononDensityMatrix, TwoQubitDensityMatrix, FOCK_DIM,
    HILBERT_DIM, TWO_QUBIT_DIM,
};
pub use quantum_acoustic_anyons::*;
pub use quantum_acoustic_waveguide::*;
pub use quantum_phonon_teleportation::*;
pub use quantum_plasmonics::{
    evaluate_plasmonic_directional_coupler, evaluate_transistor_logic, evaluate_waveguide_bend,
    run_quantum_plasmonic_benchmark, BlochVector, PlasmonicCouplerReport,
    PlasmonicMaxwellBlochSolver, PlasmonicSweepResult, QuantumPlasmonicBenchmarkReport,
    TransistorLogicReport, WaveguideBendReport,
};
pub use quantum_time_crystal::*;
pub use quantum_topological_squeezing::*;
pub use quantum_cavity_acoustomechanics::*;
pub use quantum_teleportation_waveguide::*;
pub use relay::{
    AutonomousRelaySynthesizer, CoupledRelaySolver, CoupledRelayTransientResult,
    CoupledSolverConfig, RelayBenchmarkReport, RelayBenchmarkRunner, RelaySynthesisTarget,
    SynthesizedRelayGate,
};
pub use rf::{
    constant_reactance_arc, constant_reactance_circle, constant_resistance_circle,
    format_touchstone_s2p, gamma_to_normalized_z, gamma_to_z, load_stability_circle,
    normalized_z_to_gamma, source_stability_circle, standard_reactance_values,
    standard_resistance_values, z_to_gamma, Complex64, FrequencySweep, HarmonicBalanceResult,
    HarmonicBalanceSolver, HarmonicComponent, MultiPortSSolver, MultiPortSSweepResult,
    NonLinearMetrics, NonlinearDevice, SmithCircle, SweepType, TwoPortSParameters,
};
pub use sensors::{
    AerovexCoSimPacket, AerovexPhononBridge, AerovexRigidBodyState, EskfConfig, EskfNominalState,
    FailsafeMode, FaultInjectionConfig, HilActuatorControls, HilBenchmarkReport,
    HilBenchmarkRunner, HilFlightBridge, HilGpsPacket, HilSensorPacket, Matrix15x15, Matrix3x3,
    MultiRateEskf, PhononTransducerOutput, PidGains, QuadFlightController, SensorBenchmarkReport,
    SensorBenchmarkRunner,
};
pub use skyrmion_braiding_memory::*;
pub use skyrmion_phonon_drag::*;
pub use snspd::*;
pub use space::{
    GncBenchmarkReport, GncBenchmarkRunner, GncConfig, PointingMode, SpacecraftGncSolver,
    SpacecraftState,
};
pub use sparse::{
    estimate_condition_1norm, find_markowitz_pivot, MarkowitzOptions, SparseLuFactorization,
    SparseMatrixBuilder, SparseMatrixCsc,
};
pub use spintronics::{
    ClockPhase, Cmos3nmReference, MicromagneticArray, NmlBenchmarkRunner, NmlComparisonReport,
    NmlLogicSynthesizer, SynthesizedNmlLogic, TargetNmlFunction,
};
pub use stno::{
    GiantSpinSolution, GiantSpinSolver, MacrospinState, NegfMolecularSolver, NegfTransportResult,
    StnoBenchmarkReport, StnoBenchmarkRunner, StochasticLlgSolver,
};
pub use superconducting::{
    ClassicalCmosBaseline, CoprocessorComparisonReport, CryoDecoderEngine, CryoQecDecoder,
    DecodingResult, HybridCoprocessorBenchmarkRunner, OpticalInterconnect, QecCorrection,
    SoenNetwork, SoenSpikeRecord, TravelingOpticalPacket,
};
pub use superconducting_spintronics::{
    BdgHamiltonianSolver, BdgSolution, CryoSpintronicBenchmarkReport,
    CryoSpintronicBenchmarkRunner, LindbladTrajectorySolver, TopologicalQubitDensityMatrix,
    QUANTUM_CONDUCTANCE_G0,
};
pub use synthesis::{
    SynthesisConfig, SynthesizedCandidate, TopologyEvolver, TransientGateVerifier,
    TransientVerificationResult,
};
pub use topological::{
    BenchmarkComparisonReport, BraidingStepResult, FermionParitySolver, MajoranaBraidingSolver,
    ParityTrackingReport, TopologicalBenchmarkRunner,
};
pub use topological_acoustic_axion::*;
pub use topological_soliton_comb::*;
pub use topological_weyl_acoustics::*;
pub use topological_majorana_braiding::*;
pub use topological_chern_circulator::*;
pub use programmable_chiral_graph::*;
pub use majorana_surface_memory::*;
pub use phononic_anyon_collider::*;
pub use quantum_acoustic_tensor_distillation::*;
pub use opto_acoustic_quantum_repeater::*;
pub use phonon_magnon_polariton::*;
pub use acoustic_holonomic_processor::*;
pub use floquet_majorana_braiding_processor::*;
pub use opto_electro_phononic_translator::*;
pub use chiral_acoustic_router::*;
pub use topological_corner_laser::*;
pub use non_hermitian_skin_amplifier::*;
pub use chiral_chern_anyon_braiding::*;
pub use acoustomagnonic_comb::*;
pub use topological_moire_polariton::*;
pub use non_hermitian_edge_soliton::*;
pub use phonon_exciton_polariton::*;
pub use floquet_synthetic_gauge::*;
pub use holonomic_quantum_processor::*;
pub use fractional_hall_parafermion::*;
pub use topological_time_crystal::*;
pub use cavity_acoustodynamical_spin::*;
pub use topological_acoustic_skyrmion::*;
pub use non_hermitian_quadrupole_laser::*;
pub use chiral_holographic_beamforming::*;
pub use phononic_superconducting_majorana::*;
pub use chiral_floquet_hall_transistor::*;
pub use chiral_frequency_bin_bell_analyzer::*;
pub use fractional_josephson_parafermion::*;
pub use acoustomagnonic_polariton_laser::*;
pub use chiral_hinge_axion_soliton::*;
pub use chiral_moire_fractional_chern::*;
pub use quantum_acoustic_spin_liquid::*;
pub use chiral_skyrmion_magnon_polaron::*;
pub use floquet_exceptional_ring_sensor::*;
pub use chiral_quantum_hall_pfaffian::*;
pub use kitaev_spin_liquid_braiding::*;
pub use topological_acoustic_fracton::*;
pub use twisted_bilayer_moire_polariton::*;
pub use tensor_gauge_monopole_sensor::*;
pub use quantum_acoustic_surface_code::*;
pub use chiral_axion_circulator::*;
pub use hotp_quadrupole_octupole_metasurface::*;
pub use twisted_bilayer_topological_superfluid::*;
pub use fractional_chern_simons_viscometer::*;
pub use moire_skyrmion_anyon_braiding::{
    MoireSkyrmionAnyonBraidingSolver, MoireSkyrmionBenchmarkResult, MoireSkyrmionBenchmarkRunner,
};
pub use hotp_axion_hinge_circulator::{
    HotpAxionHingeBenchmarkResult, HotpAxionHingeBenchmarkRunner, HotpAxionHingeCirculatorSolver,
};
pub use fibonacci_anyon_quantum_memory::{
    FibonacciAnyonQuantumMemorySolver, FibonacciBenchmarkResult, FibonacciBenchmarkRunner,
};
pub use non_hermitian_skin_octupole_laser::{
    NonHermitianSkinOctupoleLaserSolver, SkinOctupoleBenchmarkResult, SkinOctupoleBenchmarkRunner,
};
pub use fractional_qh_entanglement_swapper::{
    FractionalQHEntanglementSwapperSolver, SwapperBenchmarkResult, SwapperBenchmarkRunner,
};
pub use parafermionic_josephson_interferometer::{
    ParafermionBenchmarkResult, ParafermionBenchmarkRunner,
    ParafermionicJosephsonInterferometerSolver,
};
pub use majorana_kramers_network::{
    KramersBenchmarkResult, KramersBenchmarkRunner, MajoranaKramersNetworkSolver,
};
pub use fracton_quadrupole_router::{
    FractonBenchmarkResult, FractonBenchmarkRunner, FractonQuadrupoleRouterSolver,
};
pub use disclination_holonomic_processor::{
    DisclinationBenchmarkResult, DisclinationBenchmarkRunner, DisclinationHolonomicProcessorSolver,
};
pub use skyrmion_vortex_polariton::{
    SkyrmionVortexBenchmarkResult, SkyrmionVortexBenchmarkRunner, SkyrmionVortexPolaritonSolver,
};
pub use twist_defect_lattice::{
    TwistDefectBenchmarkResult, TwistDefectBenchmarkRunner, TwistDefectLatticeSolver,
};
pub use pfaffian_quantum_resonator::{
    PfaffianBenchmarkResult, PfaffianBenchmarkRunner, PfaffianQuantumResonatorSolver,
};
pub use axion_string_memristor::{
    AxionStringBenchmarkResult, AxionStringBenchmarkRunner, AxionStringMemristorSolver,
};
pub use skyrmion_anyonic_repeater::{
    SkyrmionAnyonicRepeaterSolver, SkyrmionRepeaterBenchmarkResult, SkyrmionRepeaterBenchmarkRunner,
};
pub use surface_code_decoder::{
    SurfaceCodeBenchmarkResult, SurfaceCodeBenchmarkRunner, SurfaceCodeDecoderSolver,
};
pub use floquet_majorana_engine::{
    FloquetMajoranaBenchmarkResult, FloquetMajoranaBenchmarkRunner, FloquetMajoranaEngineSolver,
};
pub use monopole_harmonic_teleporter::{
    MonopoleHarmonicBenchmarkResult, MonopoleHarmonicBenchmarkRunner,
    MonopoleHarmonicTeleporterSolver,
};
pub use skyrmion_neural_processor::{
    SkyrmionNeuralBenchmarkResult, SkyrmionNeuralBenchmarkRunner,
    SkyrmionNeuralProcessorSolver,
};
pub use anyonic_knot_coprocessor::{
    AnyonicKnotBenchmarkResult, AnyonicKnotBenchmarkRunner, AnyonicKnotCoprocessorSolver,
};
pub use quasicrystal_phason_router::{
    QuasicrystalPhasonRouterSolver, QuasicrystalRouterBenchmarkResult,
    QuasicrystalRouterBenchmarkRunner,
};
pub use spin_phonon_braiding::{
    SpinPhononBenchmarkResult, SpinPhononBenchmarkRunner, SpinPhononBraidingSolver,
};
pub use anyon_condensation::{
    AnyonCondensationBenchmarkResult, AnyonCondensationBenchmarkRunner, AnyonCondensationSolver,
};
pub use corner_state_memory::{
    CornerStateMemoryBenchmarkResult, CornerStateMemoryBenchmarkRunner, CornerStateMemorySolver,
};
pub use axion_polariton_soliton::{
    AxionPolaritonBenchmarkResult, AxionPolaritonBenchmarkRunner, AxionPolaritonSolver,
};
pub use fqh_interferometer::{
    FQHInterferometerBenchmarkResult, FQHInterferometerBenchmarkRunner, FQHInterferometerSolver,
};
pub use braiding_switchyard::{
    BraidingSwitchyardBenchmarkResult, BraidingSwitchyardBenchmarkRunner, BraidingSwitchyardSolver,
};
pub use surface_code_transceiver::{
    SurfaceCodeTransceiverBenchmarkResult, SurfaceCodeTransceiverBenchmarkRunner,
    SurfaceCodeTransceiverSolver,
};
pub use hyperbolic_crystallizer::{
    HyperbolicCrystallizerBenchmarkResult, HyperbolicCrystallizerBenchmarkRunner,
    HyperbolicCrystallizerSolver,
};
pub use majorana_transmon_hybrid::{
    MajoranaTransmonBenchmarkResult, MajoranaTransmonBenchmarkRunner,
    MajoranaTransmonSolver,
};
pub use anyonic_neural_synapse::{
    AnyonicNeuralBenchmarkResult, AnyonicNeuralBenchmarkRunner,
    AnyonicNeuralSolver,
};
pub use chern_heat_engine::{
    ChernHeatEngineBenchmarkResult, ChernHeatEngineBenchmarkRunner,
    ChernHeatEngineSolver,
};
pub use optomechanical_switchyard::{
    OptomechanicalSwitchyardBenchmarkResult, OptomechanicalSwitchyardBenchmarkRunner,
    OptomechanicalSwitchyardSolver,
};
pub use saw_soliton_routing::{
    SawSolitonRoutingBenchmarkResult, SawSolitonRoutingBenchmarkRunner,
    SawSolitonRoutingSolver,
};
pub use spin_optomechanical_bridge::{
    SpinOptomechanicalBridgeBenchmarkResult, SpinOptomechanicalBridgeBenchmarkRunner,
    SpinOptomechanicalBridgeSolver,
};
pub use braiding_circuit_compiler::{
    BraidingCircuitCompilerBenchmarkResult, BraidingCircuitCompilerBenchmarkRunner,
    BraidingCircuitCompilerSolver,
};
pub use visual_studio_engine::{
    VisualStudioBenchmarkResult, VisualStudioBenchmarkRunner,
    VisualStudioEngineSolver,
};
pub use collaboration_fabric::{
    CollaborationFabricBenchmarkResult, CollaborationFabricBenchmarkRunner,
    CollaborationFabricSolver,
};
pub use gpu_tensor_mesh::{
    GpuTensorMeshBenchmarkResult, GpuTensorMeshBenchmarkRunner,
    GpuTensorMeshSolver,
};
pub use distributed_mesh::{
    DistributedMeshBenchmarkResult, DistributedMeshBenchmarkRunner,
    DistributedMeshSolver,
};
pub use neural_circuit_copilot::{
    NeuralCircuitCopilotBenchmarkResult, NeuralCircuitCopilotBenchmarkRunner,
    NeuralCircuitCopilotSolver,
};
pub use holographic_telemetry::{
    HolographicTelemetryBenchmarkResult, HolographicTelemetryBenchmarkRunner,
    HolographicTelemetrySolver,
};
pub use generative_diffusion::{
    GenerativeDiffusionBenchmarkResult, GenerativeDiffusionBenchmarkRunner,
    GenerativeDiffusionSolver,
};
pub use mask_tapeout::{
    MaskTapeoutBenchmarkResult, MaskTapeoutBenchmarkRunner,
    MaskTapeoutSolver,
};
pub use cryo_testbed::{
    CryoTestbedBenchmarkResult, CryoTestbedBenchmarkRunner,
    CryoTestbedSolver,
};
pub use quantum_digital_twin::{
    QuantumDigitalTwinBenchmarkResult, QuantumDigitalTwinBenchmarkRunner,
    QuantumDigitalTwinSolver,
};
pub use cloud_deployment::{
    CloudDeploymentBenchmarkResult, CloudDeploymentBenchmarkRunner,
    CloudDeploymentSolver,
};
pub use quantum_transceiver::{
    QuantumTransceiverBenchmarkResult, QuantumTransceiverBenchmarkRunner,
    QuantumTransceiverSolver,
};
pub use molecular_spintronics::{
    MolecularSpintronicsBenchmarkResult, MolecularSpintronicsBenchmarkRunner,
    MolecularSpintronicsSolver,
};
pub use superradiance_laser::{
    SuperradianceLaserBenchmarkResult, SuperradianceLaserBenchmarkRunner,
    SuperradianceLaserSolver,
};
pub use topological_axion::{
    TopologicalAxionBenchmarkResult, TopologicalAxionBenchmarkRunner,
    TopologicalAxionSolver,
};
pub use exceptional_surface::{
    ExceptionalSurfaceBenchmarkResult, ExceptionalSurfaceBenchmarkRunner,
    ExceptionalSurfaceSolver,
};
pub use floquet_anyon::{
    FloquetAnyonBenchmarkResult, FloquetAnyonBenchmarkRunner,
    FloquetAnyonSolver,
};
pub use skyrmionic_memory::{
    SkyrmionicMemoryBenchmarkResult, SkyrmionicMemoryBenchmarkRunner,
    SkyrmionicMemorySolver,
};
pub use quadrupole_qubit::{
    QuadrupoleQubitBenchmarkResult, QuadrupoleQubitBenchmarkRunner,
    QuadrupoleQubitSolver,
};
pub use spin_valley::{
    SpinValleyBenchmarkResult, SpinValleyBenchmarkRunner,
    SpinValleySolver,
};
pub use holonomic_quantum::{
    HolonomicQuantumBenchmarkResult, HolonomicQuantumBenchmarkRunner,
    HolonomicQuantumSolver,
};
pub use acoustomagnonic_squeezing::{
    AcoustomagnonicSqueezingBenchmarkResult, AcoustomagnonicSqueezingBenchmarkRunner,
    AcoustomagnonicSqueezingSolver,
};
pub use tripartite_router::{
    TripartiteRouterBenchmarkResult, TripartiteRouterBenchmarkRunner,
    TripartiteRouterSolver,
};
pub use acoustoelectric_transistor::{
    AcoustoelectricTransistorBenchmarkResult, AcoustoelectricTransistorBenchmarkRunner,
    AcoustoelectricTransistorSolver,
};
pub use teleportation_network::{
    TeleportationNetworkBenchmarkResult, TeleportationNetworkBenchmarkRunner,
    TeleportationNetworkSolver,
};
pub use valley_heat_pump::{
    ValleyHeatPumpBenchmarkResult, ValleyHeatPumpBenchmarkRunner,
    ValleyHeatPumpSolver,
};
pub use majorana_braiding_processor::*;
pub use acoustomagnonic_haloscope::*;
pub use polariton_quantum_memory::*;
pub use chiral_phonon_magnon_isolator::*;
pub use acoustically_levitated_nanoparticle::*;
pub use quantum_dot_spin_shuttle::*;
pub use flux_qubit_coupler::*;
pub use acoustic_frequency_synthesizer::*;
pub use magnon_phonon_repeater::*;
pub use levitated_diamond_magnetometer::*;
pub use skyrmion_synaptic_router::*;
pub use topological_polariton_synapse::*;
pub use acoustic_snspd_detector::*;
pub use superconducting_quatrit::*;
pub use ultracold_fermi_gas_sensor::*;
pub use anyon_fusion_synthesizer::*;
pub use bec_soliton_interferometer::*;
pub use axion_magnon_memory::*;
pub use superconducting_anyon_interferometer::*;
pub use levitated_superconducting_qubit::*;
pub use spin_orbit_majorana_qubit::*;
pub use anyon_braiding_processor::*;
pub use levitated_qubit_teleporter::*;
pub use parafermion_braiding_router::*;
pub use levitated_qubit_network::*;
pub use spin_valley_polariton::*;
pub use skyrmion_majorana_crossbar::*;
pub use parafermion_surface_code::*;
pub use axion_polariton_transceiver::*;
pub use superconducting_ququint::*;
pub use topological_valley_hall_router::*;
pub use phonon_magnon_polariton_comb::*;
pub use quantum_metamaterial_transceiver::*;
pub use levitated_nanodiamond_spin_sensor::*;
pub use majorana_parafermion_hybrid::*;
pub use quantum_metamaterial_beamformer::*;
pub use axion_polariton_beam_splitter::*;
pub use floquet_chern_isolator::*;
pub use valley_chiral_polariton_splitter::*;
pub use axion_magnon_polariton_isolator::*;
pub use axion_polariton_photonic_isolator::*;
pub use superconducting_quoctit::*;
pub use axion_polariton_circulator::*;
pub use quantum_metamaterial_polariton_laser::*;
pub use floquet_chern_parafermion_router::*;
pub use fractional_chern_anyon_synthesizer::*;
pub use skyrmion_polariton_transceiver::*;
pub use majorana_parafermion_lattice::*;
pub use floquet_chern_photonic_isolator::*;
pub use superconducting_quoctit_crossbar::*;
pub use floquet_chern_parafermion_transceiver::*;
pub use quantum_metamaterial_multiplexer::*;
pub use superconducting_quoctit_processor::*;
pub use fqh_pfaffian_router::*;
pub use floquet_parafermion_laser::*;
pub use skyrmion_majorana_transceiver::*;
pub use floquet_parafermion_comb::*;
pub use floquet_parafermion_memory::*;
pub use skyrmion_parafermion_transceiver::*;
pub use fqh_moore_read_processor::*;
pub use skyrmion_majorana_memory::*;
pub use fqh_moore_read_crossbar::*;
pub use skyrmion_parafermion_memory::*;
pub use skyrmion_parafermion_processor::*;
pub use fqh_moore_read_transceiver::*;
pub use floquet_parafermion_repeater::*;
pub use fqh_moore_read_memory::*;
pub use floquet_parafermion_crossbar::*;
pub use floquet_parafermion_processor::*;
pub use moore_read_logic_crossbar::*;
pub use sensitivity::{
    AdjointSensitivityEngine, CircuitParameter, CornerEvaluation, CornerType, ObjectiveKind,
    SensitivityResult, WorstCaseOptimizer, WorstCaseSummary,
};
pub use transient::{
    evaluate_tr_bdf2_lte, solve_transient, CapacitorCompanion, InductorCompanion,
    IntegrationMethod, StepControlOptions, TimeWaveform, TransientOptions, TransientSolution,
    TransientStep, TR_BDF2_GAMMA,
};
pub use valley_acoustic::*;
pub use valleytronics::*;
pub use verification::{
    verify_energy_balance, verify_kcl, verify_kcl_dynamic, verify_kvl,
    verify_physical_conservation, verify_transient_kcl, verify_transient_kcl_with_context,
    EnergyBalanceReport, FullVerificationReport, KclReport, KvlReport,
};
pub use wakefield::{
    BeamBunch, BorisPusher, PwfaBenchmarkReport, PwfaBenchmarkRunner, RelativisticParticle,
    TrackingSummary, WakefieldAccelerator, ELECTRON_MASS_KG, ELEMENTARY_CHARGE, SPEED_OF_LIGHT,
    VACUUM_PERMITTIVITY,
};
pub use port_hamiltonian::*;
pub use symplectic_multirate::*;
pub use audio_dsp_synth::*;
pub use monte_carlo::*;
pub use polariton_waveguide::{
    ChiralEdgeModeSolver, ChiralLatticeDefect, ChiralTransmissionPoint, ChiralWavefunction2D,
    MultiModePolaritonDispersionSolver, PolaritonWaveguideParams,
};
pub use cluster::{
    ClusterDispatchQueue, ClusterSweepResult, NodeStatus, ProtocolError, RpcMessage,
    SimulatedClusterWorker, SimulationType, SweepSample, WorkerNodeInfo, PROTOCOL_MAGIC,
    PROTOCOL_VERSION,
};
pub use ep_sensor::{
    EpOrder, NhseLattice, NhseResult, NonHermitianHamiltonian, PtCircuitParams, PtCircuitState,
    PtPhase, RiemannBranchPoint,
};
pub use weyl_semimetal::{
    BandPoint, BeamSplitterResult, ChiralAnomalyTransport, FermiArcPoint, FermiArcSurface,
    WeylNode, WeylNodeType, WeylSemimetalModel,
};
pub use fqh_braiding::{
    AnyonModelKind, BraidGenerator, BraidSequence, Complex as FqhComplex, ComplexMatrix2x2,
    ComplexMatrix4x4, FillingFraction, FqhEdgeInterferometer, InterferometerType,
    SynthesisResult, TargetGate, TopologicalGateSynthesizer,
};
pub use jtwpa_simulator::{
    GainSpectrumPoint, JosephsonCellParams, JosephsonTransmissionLine, JtwpaParams, JtwpaSolver,
    QuantumSqueezing, RpmStubParams,
};
pub use floquet_metasurface::{
    bessel_j, FloquetMetasurfaceSolver, FloquetModulationParams, FloquetSidebandResult,
    MetasurfaceUnitCell, NonReciprocalScattering, OrbitalAngularMomentum, PolarPhaseMap,
    SyntheticGaugeField,
};
pub use non_abelian_holonomic::{
    DarkSubspace, HolonomicComplex, HolonomicComplexMatrix2x2, HolonomicGateType,
    HolonomicSynthesisResult, HolonomicTrajectorySimulation, ParameterLoop,
    TripartiteCavityParams, TripartiteCoSimulator, WilczekZeeConnection, WilsonLoopIntegrator,
};
pub use twisted_moire_superlattice::{
    AtomicRelaxationField, BilayerLatticeParams, DensityOfStates, DosPoint, HighSymmetryKPoint,
    LocalizedAcousticSoliton, MoireBandPoint, MoireBandStructure, StackingClassification,
    StrainTensor,
};
pub use acoustic_chern_circulator::{
    BerryCurvaturePoint, ChernLatticeParams, ChernLatticeState, ChiralEdgeMode, CirculatorPort,
    DefectImmunityResult, EdgeDispersionPoint, ObstacleKind, SParameterSpectrum,
    SParameterSpectrumPoint, ScatteringMatrix3x3, ThreePortCirculator,
};
pub use kerr_microcomb::{
    CombSpectrum, DetuningScanResult, LleSplitStepSolver, MicrocombRegime, MicrocombState,
    MicroresonatorParams,
};
pub use exceptional_surface::{
    ChiralDirectionalMetrics, EsComplex, EsManifoldParams, ExceptionalSurfaceArray,
    ExceptionalSurfaceHamiltonian, RiemannSheetPoint, SensorElement, SnrAnalysis,
    SurfaceEigenvalues, TransducerType,
};
pub use soti_corner_resonator::{
    BandDispersionPoint, BbhComplex, BbhHamiltonian, CornerEigenstate, CornerId,
    HighSymmetryPoint, QuadrupoleParams, SotiLattice, SotiLatticeResult, HIGH_SYMMETRY_PATH,
};
pub use lieb_lattice::{
    hermitian_eigensolver, jacobi_symmetric_eigensolver, AbCagingSimulator, CompactLocalizedState,
    DisorderResilienceResult, HighSymmetryPoint as LiebHighSymmetryPoint, LiebBandPoint,
    LiebComplex, LiebHamiltonian, LiebLattice, LiebLatticeResult, LiebParams, XorShiftRng,
    HIGH_SYMMETRY_PATH as LIEB_HIGH_SYMMETRY_PATH,
};
pub use axion_insulator::{
    AxionBandPoint, AxionComplex, AxionHamiltonian, AxionParams, AxionRodLattice, AxionRodResult,
    CliffordGamma, HingeDisorderResult, HingeEigenmode, HingeId, HingeSParameters,
    HighSymmetryPoint as AxionHighSymmetryPoint,
    AXION_HIGH_SYMMETRY_PATH,
};
pub use floquet_time_crystal::{
    EdwardsAndersonOrder, FloquetState, FloquetStateKind, FloquetTimeCrystalParams,
    FloquetUnitaryOperator, RigidityPhaseDiagram, StroboscopicTrajectory,
    SubharmonicSpectralAnalysis,
};
pub use protected_braiding_lattice::{
    BraidStep, BraidingComplex, CompiledBraidResult, CorrectionResult, MajoranaBraidingParams,
    MajoranaMode, MajoranaTargetGate, NonAbelianBraidGenerator, ParityReadout, StabilizerCheck,
    StabilizerKind, SurfaceCodeGrid, SyndromeResult, UnitaryMatrix, HBAR_J_S,
};
pub use optomechanical_squeezing::{
    FockStateDistribution, NonClassicalityMetrics, OptomechanicalSqueezingParams,
    PhononCountingResolvedSpectrum, PhononStateKind, QuadratureSqueezingSolver,
    QuadratureVariance, ResolvedPeak, WignerQuasiProbability, HBAR, SQL_VARIANCE,
};
pub use acoustic_skyrmion_router::{
    ChiralDomainWallRouter, DefectTransmissionResult, DomainWallDefect,
    SParameterPoint as SkyrmionSParameterPoint,
    SParameterSpectrum as SkyrmionSParameterSpectrum, SkyrmionLatticeParams, SkyrmionLatticeType,
    ThieleDynamics, TopologicalChargeCalculator, TrajectoryPoint, Vector3Field,
};
pub use acoustic_domain_wall_soliton::{
    DomainWallWaveguideParams, DomainWallWaveguideRouter, SineGordonParams, SineGordonSolver,
    SineGordonState, SolitonKind, WaveguideSParameters,
};
pub use valley_acoustic_multiplexer::{
    MultiplexerJunctionParams, MultiplexerSParameters, ValleyBerryCurvature, ValleyIndex,
    ValleyLatticeParams, ValleyLatticeSolver, ValleyMultiplexerSolver,
};
pub use non_hermitian_skin::{
    AcousticFunnelParams, AcousticFunnelSolver, FunnelSParameters, HatanoNelsonParams,
    NonHermitianSkinSolver, PointGapTopology,
};
pub use quadrupole_shg::{
    CornerShgModalMetrics, QuadrupoleShgParams, QuadrupoleShgSolver, ShgEmissionEngine,
    ShgEmissionMetrics, ShgEmissionParams, ShgHarmonicSpectrumPoint,
};
pub use synthetic_4d_qhe::{
    BoundaryHyperSurfaceMode, FourDimDispersionPoint, FourDimLatticeSolver, Synthetic4dParams,
    SyntheticHallEngine, SyntheticHallMetrics, SyntheticHallParams, SyntheticHarmonicPoint,
};
pub use pt_symmetric_acoustic::{
    InvisibilityEngine, InvisibilityMetrics, InvisibilityParams, PtAcousticParams, PtEigenvalue,
    PtHamiltonianSolver, PtModalMetrics, PtPhaseClassification, ScatteringSpectrumPoint,
    SpatialFieldPoint,
};
pub use acoustic_bic::{
    AcousticVortexFieldPoint, BicKind, BicLatticeParams, BicLatticeSolver, CavityVortexEngine,
    CavityVortexMetrics, CavityVortexParams, FanoTransmissionPoint, FarFieldPolarizationVector,
};
pub use euler_acoustic::{
    solve_real_symmetric_3x3, EulerCurvaturePoint, EulerEdgeTransportEngine, EulerLatticeSolver,
    EulerParams, EulerPhase, EulerRibbonMode, EulerTransportMetrics, RibbonDispersionPoint,
    RibbonParams,
};
pub use octupole_insulator::{
    BandPoint3D, CubicCornerId, DefectRobustnessPoint, HighSymmetryPoint3D, OctupoleCornerState,
    OctupoleCubicLattice, OctupoleHamiltonian, OctupoleLatticeResult, OctupoleParams,
    OctupolePhase,
};
pub use aah_quasicrystal::{
    solve_jacobi as solve_aah_jacobi, solve_symmetric_tridiagonal, AahEigenstate, AahHamiltonian,
    AahLatticeEngine, AahModelKind, AahParams, AahPhase, AahQuasicrystalMetrics, ButterflyPoint,
    GOLDEN_RATIO_CONJUGATE,
};
pub use valley_hall_vortex::{
    AcousticValley, DomainWallKind, PseudoLandauLevel, PumpingCyclePoint, ValleyDispersionPoint,
    ValleyHallParams, ValleyHallPhase, ValleyHamiltonian, ValleyRibbonMode, ValleyRibbonParams,
    ValleyRouterMetrics, VortexPumpingEngine,
};
pub use skyrmion_deflector::{
    AcousticPseudoSpin, DeflectedBeamResult, DeflectorParams, SkyrmionDeflectorEngine,
    SkyrmionDeflectorMetrics, SkyrmionProfileKind, SkyrmionTexture, SkyrmionTextureParams,
    SpinVector,
};
pub use floquet_frequency_dimension::{
    solve_jacobi_symmetric as solve_floquet_jacobi_symmetric, BoundaryModulationParams,
    Complex as FloquetComplex, FloquetBandPoint, FloquetFrequencyEngine,
    FrequencyConversionMetrics, FrequencyModeState, FrequencySolitonParams,
    FrequencyWavepacketProfile, SolitonRegime, SyntheticFrequencyLattice, SyntheticLatticeKind,
};
pub use non_hermitian_corner_laser::{
    CornerLaserComplex, HotLaserEigenmode, HotLaserModeKind, LaserEmissionMetrics,
    LaserLatticeKind as CornerLaserLatticeKind, LaserLatticeParams as CornerLaserLatticeParams,
    NonHermitianCornerLaserEngine, NonHermitianHotLattice,
};
pub use directional_radiation::{
    DieHitResult, DieLayer3D, DirectionalRadiationCoSimulator, HeavyIonSpecies,
    IncidentTrajectory, IonTrackProfile, MultiDieMbuEngine, RadiationTelemetryReport,
    ShieldingComponent, ShieldingMaterial, SpacecraftShieldingModel,
};
pub use atmospheric_neutron::{
    AtmosphericNeutronCoSimulator, AtmosphericNeutronModel, AtmosphericTelemetryReport,
    Do254DalLevel, FlightAltitude, LightningIndirectSimulator, LightningSeverityLevel,
    LightningTimeSample, LightningWaveformKind, MitigationArchitecture, ProtectionClampDevice,
    SiliconDeviceParams, SiliconReactionChannel, SiliconSpallationEngine, SolarModulation,
};
pub use thermal_vacuum::{
    CarrierFreezeoutModel, CryogenicDopantKind, CryogenicKinkModel, MicroBumpGeometry,
    OrbitalCyclingSimulator, OrbitalMissionKind, SolderAlloyKind, SubthresholdSteepeningModel,
    SurfaceCoatingKind, ThermalVacuumCoSimulator, ThermalVacuumTelemetryReport,
    VacuumRadiationModel, BOLTZMANN_K, DEEP_SPACE_SINK_KELVIN, ELEMENTARY_CHARGE_Q,
    STEFAN_BOLTZMANN,
};
pub use space_avionics_bus::{
    AfdxSwitch, AfdxVirtualLink, BusTelemetryReport, NoCMeshSimulator, NoCRouterTile,
    NoCRoutingPolicy, SpFiQoSScheduling, SpFiVirtualChannel, SpWLinkState,
    SpaceAvionicsBusCoSimulator, SpaceFibreMultiLaneLink, SpaceWireLink,
};
pub use rhbd_self_healing::{
    CoreHealthTelemetry, CoreLifecycleState, DrcViolationSeverity, ElectronicCrowbarParams,
    FlightTask, LayoutComponentKind, MigrationEvent, ParasiticThyristorParams,
    RhbdDrcRuleType, RhbdDrcViolation, RhbdLayoutComponent, RhbdLayoutGrid,
    RhbdSelfHealingCoSimulator, RhbdTelemetryReport, SelQuenchingSimulator,
    SelSimulationResult, SelTransientPoint, SelfHealingCluster, TaskCriticality,
};
pub use production_economics::{
    BomLineItem, CentralCostRegistry, EconomicsTelemetryReport, HierarchicalBom,
    PriceEntry, ProductionEconomicsCoSimulator, ProductionVolumeModel, VolumeBreakpoint,
};
pub use chiplet_packaging::{
    compute_s_parameters, evaluate_thermo_mechanics, evaluate_ucie_phy, extract_rdl_rlgc,
    extract_tsv_rlgc, ChipletPackagingCoSimulator, EyeDiagramSample, LayerMaterial,
    PackageStackGeometry, PackagingArchitecture, PackagingTelemetryReport, RdlGeometry, RdlRlgc,
    SParameterPoint, ThermalCycleParams, TsvGeometry, TsvRlgc, UcieDataRateGbps, UcieEyeMetrics,
    UciePackageType, UciePhyParams, WarpageStressReport,
};
pub use electrothermal_throttling::{
    calculate_dynamic_power, calculate_immersion_performance, calculate_leakage_current,
    calculate_leakage_power, calculate_leakage_temp_derivative, calculate_microchannel_performance,
    calculate_rohsenow_heat_flux, calculate_zuber_chf, generate_bifurcation_curve,
    generate_boiling_curve, run_closed_loop_transient, solve_thermal_equilibrium,
    BifurcationCurve, BoilingCurvePoint, CoolantFluid, CoolingArchitecture, DvfsControllerConfig,
    DynamicPowerParams, ElectrothermalCoSimulator, EquilibriumResult, ImmersionCoolingParams,
    ImmersionFluidKind, ImmersionFluidProperties, ImmersionPerformance, LeakageModelParams,
    LiquidCoolantProperties, MicrochannelParams, MicrochannelPerformance, PState,
    ThermalStabilityStatus, ThrottlingTelemetryReport, TimAgingModel, TimAgingPoint,
    TransientSimulationResult, TransientStepRecord, WorkloadProfile,
};
pub use pdn_droop::{
    calculate_pdn_impedance_profile, simulate_dynamic_droop, simulate_mitigated_droop,
    AntiResonancePeak, ClockStretchParams, Complex as PdnComplex, DecouplingCapSpec,
    DldoControllerParams, DroopStageMetrics, DynamicDroopResult, ImpedanceSpectrumPoint,
    LoadStepProfile, MitigatedTransientResult, MitigationReport, PdnDroopCoSimulator,
    PdnImpedanceProfile, PdnNetworkParams, PdnTelemetryReport, VrmModelParams,
};
pub use silicon_aging::{
    build_default_m1_to_m15_stack, calculate_bti_ac_recovery_factor, calculate_bti_dc_vth_shift,
    calculate_bti_effective_vth_shift, calculate_fit_rate, calculate_hci_gm_degradation,
    calculate_hci_vth_shift, calculate_max_lateral_field, calculate_mean_free_path_nm,
    calculate_on_current_degradation, calculate_progressive_gate_leakage_density,
    calculate_stage_delay_penalty, calculate_substrate_current,
    calculate_subthreshold_swing_degradation, calculate_t63_eta_sec,
    calculate_weibull_failure_probability, calculate_weibull_plot_w, evaluate_full_metal_stack,
    evaluate_metal_layer_em, generate_bti_trajectories, generate_hci_trajectories,
    generate_weibull_reliability_curve, BlacksEquationParams, BtiParams, HciParams,
    MetalLayerEmResult, MetalLayerId, MetalLayerProperties, SiliconAgingCoSimulator,
    SiliconAgingSnapshot, SiliconAgingTelemetryReport, SiliconAgingTimeCurves, TddbParams,
    TransistorPolarity, HOURS_PER_YEAR, K_BOLTZMANN_EV_PER_K, SECONDS_PER_YEAR,
};
pub use wafer_yield::{
    calculate_analytical_gross_dpw, classify_die_harvest, compute_wafer_economics,
    evaluate_die_process_parameters, generate_die_grid, generate_yield_curves, DefectRng,
    DefectYieldParams, DieArchitectureParams, DieGridPosition, DieHarvestStatus,
    DieProcessParameters, HarvestSkuTier, ProcessVariationFieldParams, SimulatedDie,
    WaferEconomicsParams, WaferEconomicsReport, WaferGeometryParams, WaferYieldCoSimulator,
    WaferYieldTelemetryReport, YieldCurvePoint,
};
pub use dse_optimization::{
    assign_crowding_distance, evaluate_genome, evaluate_packaging_ppac, generate_gp_slice,
    non_dominated_sort, polynomial_mutation, sbx_crossover, BayesianSlicePoint, DesignGenome,
    DseCoSimulator, DseTelemetryReport, GaussianProcessParams, GaussianProcessRegressor,
    GeneticRng, Individual, ObjectiveValues, PackagingPpacResult, PackagingTechnology,
};
pub use silicon_lifecycle::{
    encode_i3c_packet, encode_jtag_packet, encode_mctp_packet, encode_smbus_packet, AnomalyEvent,
    DigitalTwinModel, IpBlock, JtagTapState, ObservabilityMetrics, OnDieSensor, ProtocolType,
    SensorKind, SensorMesh, SensorPlacementAdvisor, SensorStatus, SiliconLifecycleCoSimulator,
    SlmTelemetryReport, SpatialFieldEvaluator, SpatialGridPoint, TelemetryBusMetrics,
    TelemetryPacket, TelemetryStreamEngine,
};
pub use wavepacket_scattering::{
    AcousticPhononMode, BarrierShape, BoundaryTransmissionResult, Complex as WavepacketComplex,
    InelasticScatteringKinematics, PotentialBarrier, SchroedingerStepper, WavepacketDiagnostics,
    WavepacketParams,
};
pub use skyrmion_reservoir::{
    EffectiveField, LlgsParams, MagneticSkyrmionTexture, PinningSite, SkyrmionGridParams,
    SpinTorqueOscillator, SpintronicReservoir, SpintronicReservoirParams, Vector3 as SkyrmionVector3,
};
pub use phonon_magnon_polariton::{
    MagnetoelasticDriveEngine, MagnetoelasticDriveParams, MagnetoelasticTrackSnapshot,
    PhononMagnonParams, PolaritonDispersionEngine, PolaritonDispersionPoint,
    QuantumTransducerSolver, SParameterSample, TransducerCouplingParams,
    GYROMAGNETIC_RATIO,
};
pub use quadrupole_parametric::{
    BbhBandPoint, BoundaryDispersionPoint, ParametricAmplifierMetrics, ParametricDriveParams,
    ParametricEdgeAmplifier, ParametricGainSample, QuadrupoleWaveguide, QuadrupoleWaveguideParams,
    ShgParams, ShgPhaseMatchSample, ShgSolver, ShgStepPoint,
};
pub use non_hermitian_sensor::{
    AcousticMagnonicMagnetometer, Complex as NhComplex, EpSensor, EpSensorParams,
    ExceptionalPointOrder, MagnetoacousticParams, MagnetometerTelemetry,
    NonHermitianLatticeParams, SkinEffectSolver,
};
pub use corner_harmonic_doubler::{
    CornerBendAngle, CornerCouplingParams, CornerEigenstate as DoublerCornerEigenstate,
    CornerId as DoublerCornerId, CornerToEdgeLattice, CornerToEdgeLatticeResult,
    CornerTopologicalRouter, DoublerParams, DoublerSteadyState, DoublerTransientPoint,
    EdgeEigenstate, HarmonicSpectrumPoint, NonlinearFrequencyDoubler, PortTelemetry, RouterParams,
    RouterTargetPort, RoutingWaveField, ScatteringMatrix,
};
pub use universal_braiding_processor::{
    ArbitraryRzRotation, AuditCriterion, BellStateKind, BraidingAuditReport, BraidingParams,
    CliffordTGateCompiler, CompiledGateResult, CrossbarMatrixRouter, CrossbarParams,
    DispersiveCavityResponse, ElementaryBraid, EntanglementSynthesizer, FermionParity,
    InterferometerParams, MajoranaZeroMode, ParitySpectrumData, QndTrajectoryTrace,
    TargetGate as UniversalTargetGate, UniversalBraidingProcessor,
};
pub use chiral_polariton_circulator::{
    ChiralPolaritonCirculator, ChiralPolaritonParams, CirculatorAuditCriterion,
    CirculatorAuditReport, CirculatorParams, Complex as CirculatorComplex,
    CryogenicIsolatorMetrics, CryogenicIsolatorParams, FloquetPolaritonDispersion,
    PolaritonBranchPoint, SParameters as PolaritonSParameters,
    ThreePortCirculator as ChiralThreePortCirculator,
};
pub use josephson_parametric_amplifier::{
    CvClusterStateParams, CvClusterStateSolver, CvEntanglementMetrics,
    JpaAuditCriterion, JpaAuditReport, JpaWaveguideParams, JosephsonInductanceModel,
    JosephsonParametricProcessor, ParametricAmplificationResponse,
    SqueezedVacuumSolver, SqueezingParams,
    WignerQuasiProbability as JpaWignerQuasiProbability,
};
pub use optomagnonic_comb::{
    AvoidedCrossingPoint, CombAuditItem, CombAuditReport, CombModeData,
    Complex as OptomagnonicComplex, JitterAnalysisParams, LlePolaritonParams,
    LlePolaritonResult, LlePolaritonSolver, OptomagnonicCombProcessor,
    PolaritonBranch, PureFft, TimingJitterMetrics, TimingJitterSolver,
    TripleResonanceParams, TripleResonanceResult, TripleResonanceSolver,
};
pub use floquet_time_crystal_sensor::{
    DistributedSensorNetwork, FloquetTimeCrystalSensorProcessor,
    FourierSpectrumData as SensorFourierSpectrumData, LocalizedDipoleResult, MagneticDipoleSource,
    MagnetometerParams, MagnetometerReadout, RigidityPlateauData as SensorRigidityPlateauData,
    SensorNetworkParams, SensorNode, StroboscopicResult as SensorStroboscopicResult,
    SubharmonicMagnetometer, TimeCrystalAuditCriterion, TimeCrystalAuditReport,
    TimeCrystalComplex, TimeCrystalDynamicsSolver, TimeCrystalParams,
};

pub use acoustic_metasurface_hologram::{
    bessel_j as hologram_bessel_j, AcousticMedium, AcousticMetasurfaceProcessor, AiryBeamParams,
    BesselAirySolver, BesselBeamParams, BesselBeamResult, GerchbergSaxtonParams, GorkovFieldPoint,
    HoloComplex as MetasurfaceHoloComplex, HologramIterationPoint, HologramSynthesisResult,
    HologramSynthesizer, HologramTargetType, MetasurfaceArray, MetasurfaceAuditItem,
    MetasurfaceAuditReport, MetasurfaceCellGeometry, MetasurfaceCellParams,
    MetasurfaceUnitCell as HolographicMetasurfaceUnitCell, TractorBeamEngine, TrapStabilityMetrics,
    TrappedParticle, UnitCellResponse as HolographicUnitCellResponse,
};

pub use majorana_surface_code::{
    BraidComplex as MajoranaBraidComplex, BraidTrajectoryStep, CodeDistance,
    CompiledBraidGate, CrossbarGeometry, DispersiveParityReadoutParams,
    DistillationMetrics, InterconnectCrossbarMetrics,
    MagicDistillationEngine, MagicDistillationParams, MajoranaBraidingCrossbar,
    MajoranaBraidingCrossbarParams, MajoranaSurfaceCodeAuditReport,
    MajoranaSurfaceCodeCoprocessor, MajoranaSurfaceCodeCriterion,
    MajoranaZeroMode as CrossbarMajoranaZeroMode,
    Mat2x2 as MajoranaMat2x2, ParityReadoutResult, ParitySpectrumPoint,
    PauliOperator, RecoveryResult, SurfaceCodePatch, SurfaceRng,
    SurfaceStabilizerCheck, SurfaceStabilizerKind, SyndromeExtractionResult,
    TargetCliffordGate, ThresholdCurvePoint,
};

pub use quantum_optomechanical_transducer::{
    QuantumOptomechanicalTransducer, ScatteringMatrixPoint, SidebandCoolingEngine,
    SidebandCoolingParams, TransductionEngine, TransductionParams, TransducerAuditItem,
    TransducerAuditReport, TransmonInterfaceEngine, TransmonInterfaceParams,
};

pub use corner_polariton_microcomb::{
    AllanDeviationPoint, CornerModeProfile, CornerPolaritonCavityEngine,
    CornerPolaritonMicrocombSynthesizer, CornerPolaritonParams, FrequencySynthesizerEngine,
    MicrocombAuditItem, MicrocombAuditReport, PhaseNoisePoint, SolitonCombPoint,
    SolitonDynamicsEngine, SolitonDynamicsParams, SolitonTemporalPoint, SynthesizerParams,
};

pub use giant_atom_qed::{
    CollectiveCouplingMatrix, EntanglementTrajectoryPoint, GiantAtomAuditReport, GiantAtomParams,
    GiantAtomProcessor, GiantAtomTopology, MultiAtomEntanglementResult, MultiAtomSystem,
    NonMarkovianDynamicsResult, NonMarkovianSolver, NonMarkovianTrajectoryPoint, ScatteringPoint,
    WaveguideScatteringSpectrum,
};

pub use chiral_edge_magnetoplasmon::{
    ChiralDispersionSolver, ChiralEdgeMagnetoplasmonRouter, ChiralEmpParams, DefectParams,
    EmpAuditCriterion, EmpAuditReport, EmpCirculator, EmpCirculatorParams, EmpDispersionPoint,
    EmpSMatrix3x3, EmpSpectrumPoint, QuantumHallRouter, QuantumHallRouterParams, RouterChannel,
    RouterTransportMetrics,
};

pub use polariton_bec_vortices::{
    BecCondensationMetrics, CondensateSpatialPoint, FringePatternPoint, GrossPitaevskiiSolver,
    JosephsonInterferometerParams, JosephsonInterferometerSolver, JosephsonSensorMetrics,
    JosephsonTrajectoryPoint, PolaritonBecAuditCriterion, PolaritonBecAuditReport,
    PolaritonBecInterferometer, PolaritonBecParams, QuantizedVortexSolver, VortexCharge,
    VortexGridPoint, VortexLatticeMetrics, VortexSuperfluidParams,
};

pub use synthetic_dimension_router::{
    ChannelRoutingPoint, SyntheticBandPoint, SyntheticDimensionAuditCriterion,
    SyntheticDimensionAuditReport, SyntheticDimensionRouter, SyntheticLatticeMetrics,
    SyntheticLatticeParams, SyntheticLatticePoint, SyntheticLatticeSolver,
    SyntheticMultiplexedRouter, SyntheticRouterMetrics, SyntheticRouterParams, WeylArcPoint,
    WeylSyntheticParams, WeylTransportMetrics, WeylTransportSolver, WeylWavepacketPoint,
};

pub use non_hermitian_skin_laser::{
    ChiralEmitterMetrics, ChiralEmitterParams, ChiralEmitterSolver, ComplexEigenPoint,
    LaserCurvePoint, NonHermitianSkinLaser, QuadrupoleSkinMetrics, QuadrupoleSkinParams,
    QuadrupoleSkinPoint, QuadrupoleSkinSolver, RadiationPatternPoint, SkinLaserAuditCriterion,
    SkinLaserAuditReport, TopologicalLaserMetrics, TopologicalLaserParams, TopologicalLaserSolver,
};

pub use moire_polariton_comb::{
    CombLinePoint, CornerMicrocombMetrics, CornerMicrocombParams, CornerMicrocombSolver,
    CornerModePoint, MoireCombAuditCriterion, MoireCombAuditReport, MoireCombBandPoint,
    MoireFlatBandMetrics, MoireFlatBandParams, MoireFlatBandSolver, MoirePolaritonComb,
    MoireSpatialPoint, PolaritonSolitonMetrics, PolaritonSolitonParams, PolaritonSolitonSolver,
    SolitonProfilePoint,
};

pub use holonomic_braiding_coprocessor::{
    ActuatorPulsePoint, BraidTrajectoryPoint, ChannelCrossbarStatus, CmosMemsMetrics,
    CmosMemsParams, CmosMemsSolver, HoloBraidingMetrics, HoloBraidingParams, HoloBraidingSolver,
    HoloComplex, HoloMajoranaMode, HoloMatrix2x2, HoloParitySpectrumPoint,
    HolonomicAuditCriterion, HolonomicAuditReport, HolonomicBraidStep, HolonomicBraidingCoprocessor,
    HolonomicGateKind, HolonomicGateMetrics, HolonomicGateParams, HolonomicGateSolver,
    HolonomicTrajectoryPoint,
};

pub use valley_chiral_isolator::{
    ChiralIsolatorParams, ChiralIsolatorSolver, ChiralSParameterPoint, MicrowavePhononTransducer,
    TransducerParams, TransducerResponsePoint, ValleyChiralAuditReport, ValleyChiralIsolator,
    ValleyChiralParams, ValleyEdgeMode, ValleyEdgeParams, ValleyHallLattice, ValleyPolarity,
};

pub use floquet_spinhall_circulator::{
    CirculatorSMatrix, CryogenicReadoutEngine, CryogenicReadoutParams, CryogenicReadoutPoint,
    FloquetCirculator, FloquetCirculatorParams, FloquetSpinHallAuditReport,
    FloquetSpinHallCirculator, FloquetSpinHallParams, SpinHallEdgeMode, SpinHallLattice,
    SpinHallParams, SpinHallPseudoSpin,
};

pub use octupole_dislocation_router::{
    ChiralDislocationConduit, CornerStateMode, DislocationMode, DislocationRouterParams,
    MultiPortDislocationRouter, OctupoleBandPoint, OctupoleDislocationAuditReport,
    OctupoleDislocationParams, OctupoleDislocationRouter, OctupoleLattice,
    OctupoleMetamaterialParams, RouterSParameterPoint, ScrewDislocationParams,
};

pub use chiral_majorana_braiding::{
    ChiralBraidGate, ChiralCliffordGateKind, ChiralDecodingResult, ChiralMajoranaAuditReport,
    ChiralMajoranaBraidingNetwork, ChiralMajoranaBraidingParams, ChiralMajoranaMode,
    ChiralMajoranaParams, ChiralMajoranaProcessor, ChiralParitySpectrumPoint, ChiralStabilizerKind,
    ChiralSurfaceDecoder, ChiralSurfaceDecoderParams, ChiralSyndromeDefect,
    ChiralTransmonParityReadout, ChiralTransmonReadoutParams,
};



