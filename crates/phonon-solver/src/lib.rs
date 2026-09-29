//! Phonon Solver: high-performance sparse linear algebra, Modified Nodal Analysis (MNA),
//! Markowitz threshold pivoting, dynamic TR-BDF2 transient solver, and physical conservation probes.

pub mod acoustic;
pub mod acoustoelectric;
pub mod assets;
pub mod cavity_spintronics;
pub mod chiral_phonon;
pub mod chiral_polariton;
pub mod cqed;
pub mod diamond_nv;
pub mod em;
pub mod error;
pub mod floquet;
pub mod fqh;
pub mod hetero;
pub mod hexagonal_majorana;
pub mod jtwpa;
pub mod kitwpa;
pub mod lidar;
pub mod magnon_bec;
pub mod mixed_signal;
pub mod mna;
pub mod moire;
pub mod molecular;
pub mod mvl;
pub mod net;
pub mod neuromorphic;
pub mod non_hermitian;
pub mod optics;
pub mod optimization;
pub mod optomechanics;
pub mod parallel;
pub mod phononic;
pub mod phononic_topological;
pub mod plasma;
pub mod quantum;
pub mod quantum_acoustic;
pub mod quantum_plasmonics;
pub mod relay;
pub mod rf;
pub mod sensors;
pub mod snspd;
pub mod space;
pub mod sparse;
pub mod spintronics;
pub mod stno;
pub mod superconducting;
pub mod superconducting_spintronics;
pub mod synthesis;
pub mod topological;
pub mod transient;
pub mod verification;
pub mod wakefield;

pub use acoustic::{
    AcousticBenchmarkReport, AcousticBenchmarkRunner, AcousticLinkSimulator, AcousticRealismTier,
    AcousticRoom, AcousticStepResult, FdtdResult,
};
pub use acoustoelectric::*;
pub use assets::{
    AcousticRayHit, AssetBenchmarkReport, AssetBenchmarkRunner, MeshInstance, MultiPhysicsBvh,
    MultiPhysicsBvhNode, MultiPhysicsScene, OpticalRayHit, RfTransmissionResult, WorldTriangle,
};
pub use cavity_spintronics::*;
pub use chiral_phonon::*;
pub use chiral_polariton::*;
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
pub use fqh::*;
pub use hetero::{
    HeteroCpuBenchmarkResult, HeteroCpuBenchmarkRunner, HeteroCpuOptimizer,
    HeteroOptimizationCandidate, PipelineTimingReport, TimingPathAnalyzer,
};
pub use hexagonal_majorana::*;
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
pub use mixed_signal::{
    solve_mixed_signal, DigitalTraceStep, MixedSignalCircuit, MixedSignalOptions,
    MixedSignalSolution,
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
    generate_lorenz63, generate_mackey_glass, generate_narma10, LiquidStateMachine, LsmConfig,
    NeuromorphicBenchmarkReport, NeuromorphicBenchmarkRunner, ReservoirSolver, TrainedReadout,
};
pub use non_hermitian::{
    LaserSimulationResult, MaxwellBlochSolver, NonHermitianEigenResult, NonHermitianEigensolver,
    TopologicalLaserBenchmarkReport, TopologicalLaserBenchmarkRunner,
};
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
pub use phononic_topological::*;
pub use plasma::{
    AlfvenFluxNode, AlfvenMhdConfig, AlfvenMhdStepper, BorisPicTracker, GradShafranovGrid,
    GradShafranovSolution, GradShafranovSolver, OrbitTopology, ParticleOrbitReport,
    TokamakBenchmarkReport, TokamakBenchmarkRunner, TokamakScenario,
};
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
pub use quantum_plasmonics::{
    evaluate_plasmonic_directional_coupler, evaluate_transistor_logic, evaluate_waveguide_bend,
    run_quantum_plasmonic_benchmark, BlochVector, PlasmonicCouplerReport,
    PlasmonicMaxwellBlochSolver, PlasmonicSweepResult, QuantumPlasmonicBenchmarkReport,
    TransistorLogicReport, WaveguideBendReport,
};
pub use relay::{
    AutonomousRelaySynthesizer, CoupledRelaySolver, CoupledRelayTransientResult,
    CoupledSolverConfig, RelayBenchmarkReport, RelayBenchmarkRunner, RelaySynthesisTarget,
    SynthesizedRelayGate,
};
pub use rf::{Complex64, TwoPortSParameters};
pub use sensors::{
    AerovexCoSimPacket, AerovexPhononBridge, AerovexRigidBodyState, EskfConfig, EskfNominalState,
    FailsafeMode, FaultInjectionConfig, HilActuatorControls, HilBenchmarkReport,
    HilBenchmarkRunner, HilFlightBridge, HilGpsPacket, HilSensorPacket, Matrix15x15, Matrix3x3,
    MultiRateEskf, PhononTransducerOutput, PidGains, QuadFlightController, SensorBenchmarkReport,
    SensorBenchmarkRunner,
};
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
pub use transient::{
    evaluate_tr_bdf2_lte, solve_transient, CapacitorCompanion, InductorCompanion,
    IntegrationMethod, StepControlOptions, TimeWaveform, TransientOptions, TransientSolution,
    TransientStep, TR_BDF2_GAMMA,
};
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
