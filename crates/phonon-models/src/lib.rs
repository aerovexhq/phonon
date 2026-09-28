//! Phonon Models: physically rigorous compact semiconductor models including
//! Shockley Diodes, sub-micron MOSFETs with BSIM3/4 physics and Ward-Dutton charge conservation,
//! and Gummel-Poon BJTs.

pub mod atomistic;
pub mod bjt;
pub mod chemistry;
pub mod common;
pub mod cryogenic;
pub mod diode;
pub mod em;
pub mod hetero;
pub mod hierarchical;
pub mod memristor;
pub mod mixed_signal;
pub mod molecular;
pub mod mosfet;
pub mod mvl;
pub mod net;
pub mod optimization;
pub mod parasitics;
pub mod phononic;
pub mod photonic;
pub mod quantum;
pub mod radiation;
pub mod relay;
pub mod simd;
pub mod spintronics;
pub mod superconducting;
pub mod surrogate;
pub mod synthesis;
pub mod tcad;
pub mod tline;
pub mod topological;

pub use atomistic::{
    CarbonNanotube, CntCharacter, ContactResistanceModel, ElectromigrationModel,
    InteratomicPotential, LennardJonesPotential, MdAtom, MolecularDynamicsSolver, MorsePotential,
    TddbPercolationModel, TmdMonolayer, WannierHamiltonian, WannierHopping, CARBON_BOND_LENGTH_M,
    GRAPHENE_HOPPING_EV, QUANTUM_CONDUCTANCE_SI, QUANTUM_RESISTANCE_CNT_OHMS,
};

pub use bjt::{BjtEvaluation, BjtModel, BjtType};
pub use chemistry::{
    BandAlignmentType, Bandstructure, CarrierMobilityParams, ChemicalMaterial,
    ChemicalMaterialBuilder, ChemicalMobility, ContactMaterial, ContactSpecies, CrystalStructure,
    Crystallography, DielectricMaterial, DielectricSpecies, DopantSpecies, DopantType,
    HeteroInterface, Silicon,
};
pub use common::{compute_vcrit, pn_junction_limit, safe_exp, smooth_max, smooth_min};
pub use cryogenic::{
    fermi_dirac_half, inverse_fermi_dirac_half, CryoMosfetModel, CryoMosfetOutput,
    CryogenicFreezeoutModel, CryogenicMobilityModel,
};
pub use diode::{DiodeEvaluation, DiodeModel};
pub use em::{
    compute_crc32, erfc, q_function, AmplifierClass, AntennaGeometry, ChannelProfile, ChannelRng,
    ChannelTap, ComplexField3D, CsmaCaConfig, CsmaCaStation, DielectricWall, DiscreteTransmitter,
    DopplerResult, EarthHorizon, EcefCoord, EmWaveSource, EnuCoord, FadingChannel, FrameType,
    FresnelCoefficients, GeodeticCoord, KnifeEdgeObstacle, MacAddress, MacFrame, ModulationScheme,
    OfdmConfig, OscillatorType, PhysicalAntenna, Polarization, RayHit, RfDielectricMaterial,
    RfNoiseModel, RfPowerAmplifier, SpaceNode, StationState, Vector3D, WifiPhyStandard,
    BOLTZMANN_CONSTANT, COSMIC_MICROWAVE_BACKGROUND_KELVIN, INTRINSIC_IMPEDANCE_VACUUM,
    IONO_DISPERSION_CONSTANT, MEAN_EARTH_RADIUS_METERS, ONE_TECU, REFERENCE_TEMP_K,
    SOLAR_DISK_DIAMETER_DEG, STANDARD_K_FACTOR, STANDARD_NOISE_TEMP_KELVIN, VACUUM_IMPEDANCE,
    VACUUM_PERMEABILITY, WGS84_A_METERS, WGS84_B_METERS, WGS84_E_SQ, WGS84_FLATTENING,
};
pub use hetero::{
    BlackElectromigrationModel, BlockAllocationMap, BlockStressReport, CpuMacroBlock,
    HeteroMaterialProperties, HeteroMaterialType, ProcessorBlockType, ProcessorFloorplan,
    RiscVFloorplanBuilder, ThermalHotspotReport, ThermalHotspotSolver, ThermoMechanicalStressModel,
};
pub use hierarchical::{HierarchicalDiodeBuilder, HierarchicalTransistorBuilder};
pub use memristor::{
    CrossbarCellType, FerroelectricFetModel, FilamentaryRramModel, MemristiveCrossbarModel,
    NeuronState, PhaseChangeMemoryModel, SpikeTimingPlasticityModel, SpikingNeuronModel,
};
pub use mixed_signal::{
    A2dBridge, D2aBridge, D2aCompanion, DFlipFlop, DigitalNetwork, LogicGate, LogicGateType,
    PeriodicClock, SarController,
};
pub use molecular::{
    invert_complex_matrix, solve_complex_linear_system, MolecularFullAdderCell,
    MolecularGateMetrics, MolecularGraphType, MolecularInverter, MolecularJunction, MolecularNand2,
    MolecularNor2, MolecularXor2, MultiBitMolecularAdder, NegfTransportSolver,
    CONDUCTANCE_QUANTUM_G0,
};
pub use mosfet::{MosfetEvaluation, MosfetModel, MosfetType};
pub use mvl::{
    CntfetTernaryModel, InterconnectComparison, InterconnectRentModel,
    InterconnectScalingEvaluator, MosfetFlavor, MosfetMvlEvaluation, MultiPeakRtdModel,
    MultiPeakRtdParams, MultiThresholdMosfet, MultiThresholdMosfetParams, Quat, RadixEfficiency,
    RtdEvaluation, TernaryFullAdderCell, TernaryGates, TernaryInverters,
    TernaryNoiseMarginAnalyzer, TernaryNoiseMargins, TfaOutput, ThermalRetentionReport, Trit,
};
pub use net::{
    compute_internet_checksum, nic_reg, ArpOperation, ArpPacket, ArpTable, CpuInstruction,
    IpProtocol, Ipv4Address, Ipv4Header, RouteEntry, RouterPort, RouterSwitch, SimulatedCpuNode,
    UdpDatagram, VirtualNic,
};
pub use optimization::{
    evaluate_transistor_fitness, AdjointRefiner, ArchitectureType, ChannelMaterial, FastRng,
    FitnessEvaluation, GeneBounds, Individual, IrdsNodeTarget, Nsga2Config, Nsga2Optimizer,
    OptContactMetal, OptGateDielectric, OptimizationTarget, ParameterSensitivities,
    RoadmapComplianceReport, TransistorGenome,
};
pub use parasitics::{RealCapacitorModel, RealInductorModel};
pub use phononic::{
    AcousticAnd, AcousticInverter, AcousticLayer, AcousticOr, AcousticWave, AcousticXor,
    BawResonator, DefectPhononicWaveguide, ElectricDisplacement, ElectricField, MbvdParameters,
    PhononicCrystal1D, PhononicFullAdder, PiezoelectricMaterial, SawResonator, VoigtStrain,
    VoigtStress, EPSILON_0,
};
pub use photonic::{
    ElectroOpticModulatorModel, EyeMetrics, EyeSample, LaserDiodeModel, LaserDiodeState,
    MicroRingResonatorModel, ModulatorType, OpticalWaveguideModel, PhotodetectorModel,
    PhotodetectorType, RingResonatorType, TelecomAnalyzer,
};
pub use quantum::{
    BandToBandTunnelingModel, Complex, DielectricTunnelingModel, GaaCrossSection, GaaNanowireModel,
    QuantumChannel1D,
};
pub use radiation::{
    DiceCell, DisplacementDamageModel, HeavyIonStrikeModel, LatchupEvaluation,
    ParasiticThyristorModel, StandardSramCell, StrikeOutcome, TmrVoter, TotalIonizingDoseModel,
    DEFAULT_E_EH_OXIDE_EV, E_EH_SILICON_JOULES, RAD_SI_TO_MEV_PER_G,
};
pub use relay::{
    AtomicRelayModel, AtomicRelayParameters, AtomicRelayState, EcmCellModel, EcmCellParameters,
    EcmCellState, EcmConductionState, EcmSwitchingMode, MultiBitRelayAdder, RelayContactState,
    RelayFullAdderCell, RelayLogicGate, RelaySwitchType, G_0, R_0,
};
pub use simd::{batch_evaluate_diodes_simd, batch_evaluate_nmos_simd, MosfetBatchOutput};
pub use spintronics::{
    LlgsConfig, LlgsSolver, MagneticMaterial, MultiBitNmlAdder, Nanomagnet, NmlAnd2,
    NmlFullAdderCell, NmlGateMetrics, NmlInverter, NmlMajority3, NmlOr2, Vec3,
};
pub use superconducting::{
    integrate_voltage_time, verify_flux_quantization, CryoOpticalEmitter, DcSquidModel,
    JosephsonRcsjModel, JtlStage, OptoToRsfqTransducer, PauliCorrection, RcsjCompanionStamp,
    RsfqAnd, RsfqDff, RsfqInverter, RsfqJtl, RsfqToOptoDriver, SfqPulse, SnspdModel, SoenMetrics,
    SoenNeuron, SuperconductingFluxLoop, SuperconductorMaterial, SurfaceCodeGeometry,
    SyndromePacket,
};
pub use surrogate::{
    fit_pce_surrogate, train_mlp_surrogate, ActivationFunction, DenseLayer, MlpTrainingConfig,
    MultilayerPerceptron, NeuralSurrogateCompanion, PceTerm, PolynomialChaosExpansion,
    SurrogateDeviceType,
};
pub use synthesis::{
    CircuitMetricsEvaluator, CircuitTopology, GateElement, GateMetrics, GateNode, MitDeviceModel,
    MitEvaluation, MitParameters, NdrDeviceModel, NdrEvaluation, NdrParameters, TruthTable,
    TruthTableRow,
};
pub use tcad::{
    extract_diode_model, extract_mosfet_model, MaterialProperties, Mesh1D,
    PoissonDriftDiffusionSolver, SemiconductorMaterial, TcadDevice, TcadDeviceBuilder,
    TcadDiodeCompanion, TcadMosfetCompanion, TcadState1D,
};
pub use tline::{
    BraninWaveHistory, CoupledMicrostripLine, LosslessTransmissionLine, LossyRlgcLine,
};
pub use topological::{
    ArmId, BdGHamiltonian, BdGSolution, MajoranaNanowire, NanowireParams, QubitState,
    TJunctionNanowireNetwork, TopologicalQubit, TunnelingConductanceModel, QUANTUM_CONDUCTANCE,
};
