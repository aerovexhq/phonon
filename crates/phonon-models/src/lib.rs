//! Phonon Models: physically rigorous compact semiconductor models including
//! Shockley Diodes, sub-micron MOSFETs with BSIM3/4 physics and Ward-Dutton charge conservation,
//! and Gummel-Poon BJTs.

pub mod acoustic;
pub mod assets;
pub mod atomistic;
pub mod bjt;
pub mod cavity_spintronics;
pub mod chemistry;
pub mod common;
pub mod cqed;
pub mod cryogenic;
pub mod diode;
pub mod em;
pub mod floquet;
pub mod fqh;
pub mod hetero;
pub mod hierarchical;
pub mod jtwpa;
pub mod lidar;
pub mod memristor;
pub mod mixed_signal;
pub mod moire;
pub mod molecular;
pub mod mosfet;
pub mod mvl;
pub mod net;
pub mod non_hermitian;
pub mod optics;
pub mod optimization;
pub mod optomechanics;
pub mod parasitics;
pub mod phononic;
pub mod phononic_topological;
pub mod photonic;
pub mod plasma;
pub mod quantum;
pub mod radiation;
pub mod relay;
pub mod sensors;
pub mod simd;
pub mod snspd;
pub mod space;
pub mod spintronics;
pub mod stno;
pub mod superconducting;
pub mod superconducting_spintronics;
pub mod surrogate;
pub mod synthesis;
pub mod tcad;
pub mod tline;
pub mod topological;
pub mod wakefield;

pub use acoustic::{
    compute_acoustic_doppler, evaluate_acoustic_field, speed_of_sound_in_air,
    AcousticDopplerResult, AcousticFieldPoint, AcousticMedium, AcousticObserver, AcousticSource,
    AcousticWall, CondenserMicrophone, MediumType, MicrophonePolarPattern, MicrophoneSignal,
    PiezoelectricMicrophone, ADIABATIC_INDEX_AIR, GAS_CONSTANT_R, MOLAR_MASS_AIR, P_ATM_SEA_LEVEL,
    P_REF_AIR, T_REF_KELVIN,
};
pub use assets::{
    create_cubesat_chassis, create_dipole_antenna, create_finned_heatsink, create_patch_antenna,
    create_quadrotor_frame, create_tactile_landing_gear, Aabb3D, AcousticProperties,
    ElectromagneticProperties, MaterialCategory, MaterialLibrary, MaterialRecord, Mesh3D,
    OpticalProperties, Submesh, ThermalMechanicalProperties, Triangle3D, Vertex3D,
};
pub use atomistic::{
    CarbonNanotube, CntCharacter, ContactResistanceModel, ElectromigrationModel,
    InteratomicPotential, LennardJonesPotential, MdAtom, MolecularDynamicsSolver, MorsePotential,
    TddbPercolationModel, TmdMonolayer, WannierHamiltonian, WannierHopping, CARBON_BOND_LENGTH_M,
    GRAPHENE_HOPPING_EV, QUANTUM_CONDUCTANCE_SI, QUANTUM_RESISTANCE_CNT_OHMS,
};

pub use bjt::{BjtEvaluation, BjtModel, BjtType};
pub use cavity_spintronics::*;
pub use chemistry::{
    BandAlignmentType, Bandstructure, CarrierMobilityParams, ChemicalMaterial,
    ChemicalMaterialBuilder, ChemicalMobility, ContactMaterial, ContactSpecies, CrystalStructure,
    Crystallography, DielectricMaterial, DielectricSpecies, DopantSpecies, DopantType,
    HeteroInterface, Silicon,
};
pub use common::{compute_vcrit, pn_junction_limit, safe_exp, smooth_max, smooth_min};
pub use cqed::{DispersiveCqedSystem, MicrowaveCavity, PurcellFilter, TransmonParams};
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
pub use floquet::*;
pub use fqh::*;
pub use hetero::{
    BlackElectromigrationModel, BlockAllocationMap, BlockStressReport, CpuMacroBlock,
    HeteroMaterialProperties, HeteroMaterialType, ProcessorBlockType, ProcessorFloorplan,
    RiscVFloorplanBuilder, ThermalHotspotReport, ThermalHotspotSolver, ThermoMechanicalStressModel,
};
pub use hierarchical::{HierarchicalDiodeBuilder, HierarchicalTransistorBuilder};
pub use jtwpa::{DispersionEngineeringParams, ParametricProcessParams, SnailElementParams};
pub use lidar::{
    AtmosphericCondition, EchoReturn, FogType, LaserPulseConfig, LaserRay, LidarScannerConfig,
    ScanningArchitecture, WAVELENGTH_1550_NM, WAVELENGTH_905_NM,
};
pub use memristor::{
    CrossbarCellType, DelayOscillatorType, DelayedFeedbackReservoir, FerroelectricFetModel,
    FilamentaryRramModel, MemristiveCrossbarModel, MemristiveNonIdealityConfig,
    MemristiveReservoir, MemristorTechnology, NeuronState, PhaseChangeMemoryModel,
    ReservoirActivation, ReservoirRng, SpikeTimingPlasticityModel, SpikingNeuronModel,
};
pub use mixed_signal::{
    A2dBridge, D2aBridge, D2aCompanion, DFlipFlop, DigitalNetwork, LogicGate, LogicGateType,
    PeriodicClock, SarController,
};
pub use moire::*;
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
pub use non_hermitian::{
    LaserRateEquationParams, PtDimerParams, PtPhaseRegime, SshLatticeParams,
    TopologicalLatticePhase,
};
pub use optics::{
    silicon_quantum_efficiency, transduce_cmos_pixel, CameraIntrinsics, CmosPixelConfig,
    OpticalCamera, PixelOutput, ShutterType, SILICON_BANDGAP_JOULES, SILICON_CUTOFF_WAVELENGTH_NM,
};
pub use optimization::{
    evaluate_transistor_fitness, AdjointRefiner, ArchitectureType, ChannelMaterial, FastRng,
    FitnessEvaluation, GeneBounds, Individual, IrdsNodeTarget, Nsga2Config, Nsga2Optimizer,
    OptContactMetal, OptGateDielectric, OptimizationTarget, ParameterSensitivities,
    RoadmapComplianceReport, TransistorGenome,
};
pub use optomechanics::{
    OmitParams, OptomechanicalParams, OptomechanicalSystemType, PonderomotiveSqueezingParams,
    SidebandCoolingParams,
};
pub use parasitics::{RealCapacitorModel, RealInductorModel};
pub use phononic::{
    AcousticAnd, AcousticInverter, AcousticLayer, AcousticOr, AcousticWave, AcousticXor,
    BawResonator, DefectPhononicWaveguide, ElectricDisplacement, ElectricField, MbvdParameters,
    PhononicCrystal1D, PhononicFullAdder, PiezoelectricMaterial, SawResonator, VoigtStrain,
    VoigtStress, EPSILON_0,
};
pub use phononic_topological::*;
pub use photonic::{
    ElectroOpticModulatorModel, EyeMetrics, EyeSample, LaserDiodeModel, LaserDiodeState,
    MicroRingResonatorModel, ModulatorType, OpticalWaveguideModel, PhotodetectorModel,
    PhotodetectorType, RingResonatorType, TelecomAnalyzer,
};
pub use plasma::{
    AlfvenWaveProperties, IcrfHeatingSource, KineticParticle, MhdFluidState, PlasmaSpecies,
    SafetyFactorProfile, SolovevEquilibrium, ThermonuclearFusion, TokamakBeta, TokamakGeometry,
    ToroidalAlfvenEigenmode, ALPHA_MASS, DEUTERON_MASS, DT_ALPHA_ENERGY_JOULES,
    DT_TOTAL_ENERGY_JOULES, EV_TO_JOULES, KEV_TO_JOULES, PLASMA_ADIABATIC_INDEX, PROTON_MASS,
    TRITON_MASS,
};
pub use quantum::{
    effective_damping_rate, effective_mechanical_frequency, is_ground_state_cooled,
    omit_probe_transmission, optical_cooperativity, optical_spring_shift,
    optomechanical_damping_rate, sideband_cooling_phonon_occupancy, BandToBandTunnelingModel,
    Complex, DielectricTunnelingModel, GaaCrossSection, GaaNanowireModel, OmitTransmissionResult,
    OptomechanicalHamiltonian, PiezoCrystalMaterial, PiezoOptomechanicalCrystal, QuantumChannel1D,
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
pub use sensors::{
    compute_ground_effect_factor, sample_imu, AirframeConfig, AllanNoiseConfig, BarometerSensor,
    CapacitiveSensorConfig, CollisionContactInput, DrydenWindModel, FlightDynamicsEngine,
    FlightDynamicsState, GpsFixType, GpsMeasurement, ImuConfig, ImuMeasurement, ImuState,
    InertiaTensor3D, LidarRangefinder, Quaternion, RotorConfig, BAROMETRIC_EXPONENT,
    DRY_AIR_MOLAR_MASS_KG_PER_MOL, STANDARD_GRAVITY_M_S2, STANDARD_SEA_LEVEL_PRESSURE_PA,
    STANDARD_SEA_LEVEL_TEMP_K, STANDARD_TEMP_LAPSE_RATE_K_PER_M, UNIVERSAL_GAS_CONSTANT,
};
pub use simd::{batch_evaluate_diodes_simd, batch_evaluate_nmos_simd, MosfetBatchOutput};
pub use snspd::*;
pub use space::{
    earth_geomagnetic_field, CatalogStar, InertiaTensor, KeplerianElements, MagneticTorquerSystem,
    OrbitalPerturbationSolver, ReactionWheel, ReactionWheelCluster, SpacecraftPhysicalProperties,
    StarTrackerCamera, StarTrackerSystem, J2_EARTH, J3_EARTH, J4_EARTH, MU_EARTH, MU_MOON, MU_SUN,
    R_EARTH,
};
pub use spintronics::{
    LlgsConfig, LlgsSolver, MagneticMaterial, MultiBitNmlAdder, Nanomagnet, NmlAnd2,
    NmlFullAdderCell, NmlGateMetrics, NmlInverter, NmlMajority3, NmlOr2, Vec3,
};
pub use stno::{GiantSpinParams, InjectionLockingParams, NegfMolecularJunctionParams, StnoParams};
pub use superconducting::{
    integrate_voltage_time, verify_flux_quantization, CryoOpticalEmitter, DcSquidModel,
    JosephsonRcsjModel, JtlStage, OptoToRsfqTransducer, PauliCorrection, RcsjCompanionStamp,
    RsfqAnd, RsfqDff, RsfqInverter, RsfqJtl, RsfqToOptoDriver, SfqPulse, SnspdModel, SoenMetrics,
    SoenNeuron, SuperconductingFluxLoop, SuperconductorMaterial, SurfaceCodeGeometry,
    SyndromePacket,
};
pub use superconducting_spintronics::{
    CooperPairSymmetry, CryoDac, CryoPll, CryoReadoutTia, CryoThermalBackaction, FermionParity,
    FourMajoranaQubit, SuperconductingSpintronicJunction, TopologicalNanowireParams,
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
pub use wakefield::{BetatronRadiation, BubbleRegime, LaserPulseParams, PlasmaChannelParams};
