//! Phonon Solver: high-performance sparse linear algebra, Modified Nodal Analysis (MNA),
//! Markowitz threshold pivoting, dynamic TR-BDF2 transient solver, and physical conservation probes.

pub mod acoustic;
pub mod em;
pub mod error;
pub mod hetero;
pub mod mixed_signal;
pub mod mna;
pub mod molecular;
pub mod mvl;
pub mod net;
pub mod optics;
pub mod optimization;
pub mod parallel;
pub mod phononic;
pub mod relay;
pub mod rf;
pub mod sparse;
pub mod spintronics;
pub mod superconducting;
pub mod synthesis;
pub mod topological;
pub mod transient;
pub mod verification;

pub use acoustic::{
    AcousticBenchmarkReport, AcousticBenchmarkRunner, AcousticLinkSimulator, AcousticRealismTier,
    AcousticRoom, AcousticStepResult, FdtdResult,
};
pub use em::{
    AntennaElectrodynamicSolver, EmBenchmarkReport, EmBenchmarkRunner, EmLinkResult,
    EmPropagationScene, EmWaveSolver, EmitterBenchmarkReport, EmitterBenchmarkRunner,
    GroundStation, LinkEvaluationParams, ObservationPoint, PacketTransmissionResult,
    ProtocolBenchmarkReport, ProtocolBenchmarkRunner, RadiationSphereResult, RfRealismTier,
    RfTierEngine, RfTransceiverSolver, SatelliteNode, SpaceLinkBudgetResult, SpaceLinkSolver,
    TierChannelResult, TransceiverLinkResult, TransientBurstResult, WifiLinkSimulator,
};
pub use error::SolverError;
pub use hetero::{
    HeteroCpuBenchmarkResult, HeteroCpuBenchmarkRunner, HeteroCpuOptimizer,
    HeteroOptimizationCandidate, PipelineTimingReport, TimingPathAnalyzer,
};
pub use mixed_signal::{
    solve_mixed_signal, DigitalTraceStep, MixedSignalCircuit, MixedSignalOptions,
    MixedSignalSolution,
};
pub use mna::{
    assemble_mna_dc, solve_dc_linear, solve_dc_non_linear, DcSolution, MnaSystem, ModelContext,
    NewtonOptions, SolverOptions,
};
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
pub use optics::{
    AabbBox, CheckerPlane, LightSource, OffscreenPerceptionEngine, OpticalBenchmarkReport,
    OpticalBenchmarkRunner, OpticalRealismTier, PerceptionFrame, RayHit, SceneObject, Sphere,
    SyntheticScene,
};
pub use optimization::{EngineConfig, InverseDesignEngine};
pub use parallel::{
    solve_torn_dc, CircuitSoA, DiakopticsSolver, DiodeSoA, MosfetSoA, PartitionedCircuit,
    ResistorSoA, SubcircuitLocalSolution,
};
pub use phononic::{
    AcousticGateVerificationResult, AcousticLogicSolver, AcousticWaveformTrace, ContinuumNode,
    ContinuumSolver2D, PhononicBenchmarkReport, PhononicBenchmarkRunner,
};
pub use relay::{
    AutonomousRelaySynthesizer, CoupledRelaySolver, CoupledRelayTransientResult,
    CoupledSolverConfig, RelayBenchmarkReport, RelayBenchmarkRunner, RelaySynthesisTarget,
    SynthesizedRelayGate,
};
pub use rf::{Complex64, TwoPortSParameters};
pub use sparse::{
    estimate_condition_1norm, find_markowitz_pivot, MarkowitzOptions, SparseLuFactorization,
    SparseMatrixBuilder, SparseMatrixCsc,
};
pub use spintronics::{
    ClockPhase, Cmos3nmReference, MicromagneticArray, NmlBenchmarkRunner, NmlComparisonReport,
    NmlLogicSynthesizer, SynthesizedNmlLogic, TargetNmlFunction,
};
pub use superconducting::{
    ClassicalCmosBaseline, CoprocessorComparisonReport, CryoDecoderEngine, CryoQecDecoder,
    DecodingResult, HybridCoprocessorBenchmarkRunner, OpticalInterconnect, QecCorrection,
    SoenNetwork, SoenSpikeRecord, TravelingOpticalPacket,
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
