//! Structured facts extracted from datasheets.
//!
//! Uses PhysicalQuantity for typed, dimensionally-correct engineering values.

use eak_units::{PhysicalQuantity, Tolerance, Unit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kind of fact that can be extracted from a datasheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactKind {
    // Electrical parameters
    Voltage,
    Current,
    Resistance,
    Capacitance,
    Inductance,
    Power,
    Frequency,
    Temperature,
    TemperatureMin,
    TemperatureMax,

    // Component identity
    Manufacturer,
    Mpn,
    Package,
    LifecycleStatus,

    // Other parameters
    Tolerance,
    Dimension,
    Gain,
    Bandwidth,
    SlewRate,
    Noise,
    Cmrr,
    Psrr,
    InputOffsetVoltage,
    InputBiasCurrent,
    QuiescentCurrent,
    OutputCurrent,
    RdsOn,
    GateThresholdVoltage,
    GateCharge,
    CapacitanceIss,
    CapacitanceOss,
    CapacitanceRss,
    TurnOnDelay,
    RiseTime,
    TurnOffDelay,
    FallTime,
    ReverseRecoveryTime,
    JunctionCapacitance,
    LeakageCurrent,
    ForwardVoltage,
    ReverseVoltage,
    ZenerVoltage,
    ZenerImpedance,
    ClampingVoltage,
    PeakPulseCurrent,
    BreakdownVoltage,
    ReverseWorkingVoltage,
    LuminousIntensity,
    DominantWavelength,
    ViewingAngle,
    ColorTemperature,
    Esr,
    Esl,
    RippleCurrentRating,
    Dcr,
    SaturationCurrent,
    RmsCurrent,
    Srf,
    QFactor,
    LoadCapacitance,
    DriveLevel,
    FrequencyStability,
    Aging,
    DropoutVoltage,
    LineRegulation,
    LoadRegulation,
    OutputNoise,
    ShutdownCurrent,
    CurrentLimit,
    ThermalShutdown,
    Efficiency,
    SwitchingFrequency,
    PropagationDelay,
    OutputDriveCurrent,
    InputVoltageHigh,
    InputVoltageLow,
    OutputVoltageHigh,
    OutputVoltageLow,
    MaxFrequency,
    AdditiveJitter,
    OutputSkew,
    DutyCycleDistortion,
    CoreFrequency,
    FlashSize,
    RamSize,
    GpioCount,
    AdcChannels,
    AdcResolution,
    DacChannels,
    DacResolution,
    TimerCount,
    UartCount,
    SpiCount,
    I2cCount,
    CanCount,
    Density,
    Organization,
    Interface,
    MaxFrequencyMemory,
    SupplyVoltageMin,
    SupplyVoltageMax,
    ReadCurrent,
    WriteCurrent,
    StandbyCurrent,
    PageSize,
    SectorSize,
    BlockSize,
    EraseTime,
    WriteTime,
    EnduranceCycles,
    DataRetention,
    Protocol,
    MaxDataRate,
    VoltageLevelsMin,
    VoltageLevelsMax,
    Channels,
    IsolationVoltage,
    Cmti,
    EsdProtection,
    BufferSize,
    MeasurementRangeMin,
    MeasurementRangeMax,
    Accuracy,
    Resolution,
    Sensitivity,
    NoiseDensity,
    OutputInterface,
    CurrentConsumptionActive,
    CurrentConsumptionSleep,
    CurrentConsumptionShutdown,
    ResponseTime,
    TxPower,
    RxSensitivity,
    ModulationSchemes,
    ChannelBandwidth,
    AntennaInterface,
    CurrentTx,
    CurrentRx,
    CurrentSleep,
    LinkBudget,
    Snr,
    ThdN,
    DynamicRange,
    ChannelCount,
    MasterClock,
    HeadphoneAmpPower,
    SpeakerAmpPower,
    MicBias,
    WorkingVoltage,
    BreakdownVoltageProtection,
    ClampingVoltageProtection,
    PeakPulseCurrentProtection,
    PeakPulsePower,
    HoldCurrent,
    TripCurrent,
    TripTime,
    ResponseTimeProtection,
    LeakageCurrentProtection,
    CapacitanceProtection,
    ConnectorType,
    Pitch,
    Positions,
    Rows,
    CurrentPerContact,
    VoltageRating,
    MatingCycles,
    MountingStyle,
    Orientation,
    LockingMechanism,
    Shielding,
    ImpedanceControlled,
    DifferentialPairs,
    DeviceType,
    ActuationForce,
    ActuationTravel,
    MechanicalLife,
    ElectricalLife,
    ContactRatingVoltage,
    ContactRatingCurrent,
    CoilVoltage,
    CoilPower,
    ContactForm,
    ResolutionEncoder,
    Airflow,
    StaticPressure,
    Torque,
    Speed,
    // Generic/other
    Other,
}

/// A parameter fact with typed PhysicalQuantity value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterFact {
    /// The typed physical value
    pub value: PhysicalQuantity,
    /// Optional tolerance on the value
    pub tolerance: Option<Tolerance>,
    /// Test/operating conditions (e.g., "Vcc=5V, Ta=25°C")
    pub conditions: Vec<String>,
}

impl ParameterFact {
    /// Create a new parameter fact.
    pub fn new(value: PhysicalQuantity) -> Self {
        Self {
            value,
            tolerance: None,
            conditions: Vec::new(),
        }
    }

    /// Create with tolerance.
    pub fn with_tolerance(value: PhysicalQuantity, tolerance: Tolerance) -> Self {
        Self {
            value,
            tolerance: Some(tolerance),
            conditions: Vec::new(),
        }
    }

    /// Add a condition.
    pub fn with_condition(mut self, condition: String) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Check if this fact has the same dimension as another.
    pub fn same_dimension(&self, other: &Self) -> bool {
        self.value.dimension() == other.value.dimension()
    }

    /// Compare values (requires same dimension).
    pub fn compare(&self, other: &Self) -> Result<std::cmp::Ordering, eak_units::UnitError> {
        self.value.try_compare(&other.value)
    }

    /// Check if values are the same (within epsilon).
    pub fn same_value(&self, other: &Self) -> Result<bool, eak_units::UnitError> {
        self.value.same_value(&other.value)
    }
}

/// Provenance information for an extracted fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceInfo {
    /// Asset database ID
    pub asset_id: i64,
    /// SHA-256 of the source asset file
    pub asset_sha256: String,
    /// Page number where fact was found (1-indexed)
    pub page: usize,
    /// Extracted text snippet containing the fact
    pub extracted_text: String,
    /// Parser version used
    pub parser_version: String,
    /// When the fact was extracted
    pub extracted_at: String,
    /// Whether this fact has been verified by an engineer
    pub verified: bool,
    /// Whether this fact was manually corrected
    pub manually_corrected: bool,
    /// Notes if manually corrected
    pub correction_notes: Option<String>,
}

/// A complete datasheet fact with provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasheetFact {
    /// Database ID (assigned on insert)
    pub id: Option<i64>,
    /// Asset this fact came from
    pub asset_id: i64,
    /// Kind of fact
    pub kind: FactKind,
    /// The parameter value with units and tolerance
    pub parameter: ParameterFact,
    /// Full provenance chain
    pub provenance: ProvenanceInfo,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

impl DatasheetFact {
    /// Create a new datasheet fact.
    pub fn new(asset_id: i64, kind: FactKind, parameter: ParameterFact) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: None,
            asset_id,
            kind,
            parameter,
            provenance: ProvenanceInfo {
                asset_id,
                asset_sha256: String::new(),
                page: 0,
                extracted_text: String::new(),
                parser_version: String::new(),
                extracted_at: now.clone(),
                verified: false,
                manually_corrected: false,
                correction_notes: None,
            },
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Get a human-readable description of the fact kind.
    pub fn kind_description(kind: FactKind) -> &'static str {
        match kind {
            FactKind::Voltage => "Voltage",
            FactKind::Current => "Current",
            FactKind::Resistance => "Resistance",
            FactKind::Capacitance => "Capacitance",
            FactKind::Inductance => "Inductance",
            FactKind::Power => "Power",
            FactKind::Frequency => "Frequency",
            FactKind::Temperature => "Temperature",
            FactKind::TemperatureMin => "Minimum Temperature",
            FactKind::TemperatureMax => "Maximum Temperature",
            FactKind::Manufacturer => "Manufacturer",
            FactKind::Mpn => "Manufacturer Part Number",
            FactKind::Package => "Package",
            FactKind::LifecycleStatus => "Lifecycle Status",
            FactKind::Tolerance => "Tolerance",
            FactKind::Dimension => "Dimension",
            FactKind::Gain => "Gain",
            FactKind::Bandwidth => "Bandwidth",
            FactKind::SlewRate => "Slew Rate",
            FactKind::Noise => "Noise",
            FactKind::Cmrr => "CMRR",
            FactKind::Psrr => "PSRR",
            FactKind::InputOffsetVoltage => "Input Offset Voltage",
            FactKind::InputBiasCurrent => "Input Bias Current",
            FactKind::QuiescentCurrent => "Quiescent Current",
            FactKind::OutputCurrent => "Output Current",
            FactKind::RdsOn => "RDS(on)",
            FactKind::GateThresholdVoltage => "Gate Threshold Voltage",
            FactKind::GateCharge => "Gate Charge",
            FactKind::CapacitanceIss => "Input Capacitance (Ciss)",
            FactKind::CapacitanceOss => "Output Capacitance (Coss)",
            FactKind::CapacitanceRss => "Reverse Transfer Capacitance (Crss)",
            FactKind::TurnOnDelay => "Turn-On Delay",
            FactKind::RiseTime => "Rise Time",
            FactKind::TurnOffDelay => "Turn-Off Delay",
            FactKind::FallTime => "Fall Time",
            FactKind::ReverseRecoveryTime => "Reverse Recovery Time",
            FactKind::JunctionCapacitance => "Junction Capacitance",
            FactKind::LeakageCurrent => "Leakage Current",
            FactKind::ForwardVoltage => "Forward Voltage",
            FactKind::ReverseVoltage => "Reverse Voltage",
            FactKind::ZenerVoltage => "Zener Voltage",
            FactKind::ZenerImpedance => "Zener Impedance",
            FactKind::ClampingVoltage => "Clamping Voltage",
            FactKind::PeakPulseCurrent => "Peak Pulse Current",
            FactKind::BreakdownVoltage => "Breakdown Voltage",
            FactKind::ReverseWorkingVoltage => "Reverse Working Voltage",
            FactKind::LuminousIntensity => "Luminous Intensity",
            FactKind::DominantWavelength => "Dominant Wavelength",
            FactKind::ViewingAngle => "Viewing Angle",
            FactKind::ColorTemperature => "Color Temperature",
            FactKind::Esr => "ESR",
            FactKind::Esl => "ESL",
            FactKind::RippleCurrentRating => "Ripple Current Rating",
            FactKind::Dcr => "DCR",
            FactKind::SaturationCurrent => "Saturation Current",
            FactKind::RmsCurrent => "RMS Current",
            FactKind::Srf => "Self-Resonant Frequency",
            FactKind::QFactor => "Q Factor",
            FactKind::LoadCapacitance => "Load Capacitance",
            FactKind::DriveLevel => "Drive Level",
            FactKind::FrequencyStability => "Frequency Stability",
            FactKind::Aging => "Aging",
            FactKind::DropoutVoltage => "Dropout Voltage",
            FactKind::LineRegulation => "Line Regulation",
            FactKind::LoadRegulation => "Load Regulation",
            FactKind::OutputNoise => "Output Noise",
            FactKind::ShutdownCurrent => "Shutdown Current",
            FactKind::CurrentLimit => "Current Limit",
            FactKind::ThermalShutdown => "Thermal Shutdown Temperature",
            FactKind::Efficiency => "Efficiency",
            FactKind::SwitchingFrequency => "Switching Frequency",
            FactKind::PropagationDelay => "Propagation Delay",
            FactKind::OutputDriveCurrent => "Output Drive Current",
            FactKind::InputVoltageHigh => "Input Voltage High (VIH)",
            FactKind::InputVoltageLow => "Input Voltage Low (VIL)",
            FactKind::OutputVoltageHigh => "Output Voltage High (VOH)",
            FactKind::OutputVoltageLow => "Output Voltage Low (VOL)",
            FactKind::MaxFrequency => "Maximum Frequency",
            FactKind::AdditiveJitter => "Additive Jitter",
            FactKind::OutputSkew => "Output Skew",
            FactKind::DutyCycleDistortion => "Duty Cycle Distortion",
            FactKind::CoreFrequency => "Core Frequency",
            FactKind::FlashSize => "Flash Size",
            FactKind::RamSize => "RAM Size",
            FactKind::GpioCount => "GPIO Count",
            FactKind::AdcChannels => "ADC Channels",
            FactKind::AdcResolution => "ADC Resolution",
            FactKind::DacChannels => "DAC Channels",
            FactKind::DacResolution => "DAC Resolution",
            FactKind::TimerCount => "Timer Count",
            FactKind::UartCount => "UART Count",
            FactKind::SpiCount => "SPI Count",
            FactKind::I2cCount => "I2C Count",
            FactKind::CanCount => "CAN Count",
            FactKind::Density => "Memory Density",
            FactKind::Organization => "Memory Organization",
            FactKind::Interface => "Interface",
            FactKind::MaxFrequencyMemory => "Max Frequency",
            FactKind::SupplyVoltageMin => "Min Supply Voltage",
            FactKind::SupplyVoltageMax => "Max Supply Voltage",
            FactKind::ReadCurrent => "Read Current",
            FactKind::WriteCurrent => "Write Current",
            FactKind::StandbyCurrent => "Standby Current",
            FactKind::PageSize => "Page Size",
            FactKind::SectorSize => "Sector Size",
            FactKind::BlockSize => "Block Size",
            FactKind::EraseTime => "Erase Time",
            FactKind::WriteTime => "Write Time",
            FactKind::EnduranceCycles => "Endurance Cycles",
            FactKind::DataRetention => "Data Retention",
            FactKind::Protocol => "Protocol",
            FactKind::MaxDataRate => "Max Data Rate",
            FactKind::VoltageLevelsMin => "Min Voltage Level",
            FactKind::VoltageLevelsMax => "Max Voltage Level",
            FactKind::Channels => "Channels",
            FactKind::IsolationVoltage => "Isolation Voltage",
            FactKind::Cmti => "CMTI",
            FactKind::EsdProtection => "ESD Protection",
            FactKind::BufferSize => "Buffer Size",
            FactKind::MeasurementRangeMin => "Min Measurement Range",
            FactKind::MeasurementRangeMax => "Max Measurement Range",
            FactKind::Accuracy => "Accuracy",
            FactKind::Resolution => "Resolution",
            FactKind::Sensitivity => "Sensitivity",
            FactKind::NoiseDensity => "Noise Density",
            FactKind::Bandwidth => "Bandwidth",
            FactKind::OutputInterface => "Output Interface",
            FactKind::CurrentConsumptionActive => "Active Current Consumption",
            FactKind::CurrentConsumptionSleep => "Sleep Current Consumption",
            FactKind::CurrentConsumptionShutdown => "Shutdown Current Consumption",
            FactKind::ResponseTime => "Response Time",
            FactKind::TxPower => "TX Power",
            FactKind::RxSensitivity => "RX Sensitivity",
            FactKind::ModulationSchemes => "Modulation Schemes",
            FactKind::ChannelBandwidth => "Channel Bandwidth",
            FactKind::AntennaInterface => "Antenna Interface",
            FactKind::CurrentTx => "TX Current",
            FactKind::CurrentRx => "RX Current",
            FactKind::CurrentSleep => "Sleep Current",
            FactKind::LinkBudget => "Link Budget",
            FactKind::Snr => "SNR",
            FactKind::ThdN => "THD+N",
            FactKind::DynamicRange => "Dynamic Range",
            FactKind::ChannelCount => "Channel Count",
            FactKind::MasterClock => "Master Clock",
            FactKind::HeadphoneAmpPower => "Headphone Amp Power",
            FactKind::SpeakerAmpPower => "Speaker Amp Power",
            FactKind::MicBias => "Mic Bias",
            FactKind::WorkingVoltage => "Working Voltage",
            FactKind::BreakdownVoltageProtection => "Breakdown Voltage",
            FactKind::ClampingVoltageProtection => "Clamping Voltage",
            FactKind::PeakPulseCurrentProtection => "Peak Pulse Current",
            FactKind::PeakPulsePower => "Peak Pulse Power",
            FactKind::HoldCurrent => "Hold Current",
            FactKind::TripCurrent => "Trip Current",
            FactKind::TripTime => "Trip Time",
            FactKind::ResponseTimeProtection => "Response Time",
            FactKind::LeakageCurrentProtection => "Leakage Current",
            FactKind::CapacitanceProtection => "Capacitance",
            FactKind::ConnectorType => "Connector Type",
            FactKind::Pitch => "Pitch",
            FactKind::Positions => "Positions",
            FactKind::Rows => "Rows",
            FactKind::CurrentPerContact => "Current Per Contact",
            FactKind::VoltageRating => "Voltage Rating",
            FactKind::MatingCycles => "Mating Cycles",
            FactKind::MountingStyle => "Mounting Style",
            FactKind::Orientation => "Orientation",
            FactKind::LockingMechanism => "Locking Mechanism",
            FactKind::Shielding => "Shielding",
            FactKind::ImpedanceControlled => "Impedance Controlled",
            FactKind::DifferentialPairs => "Differential Pairs",
            FactKind::DeviceType => "Device Type",
            FactKind::ActuationForce => "Actuation Force",
            FactKind::ActuationTravel => "Actuation Travel",
            FactKind::MechanicalLife => "Mechanical Life",
            FactKind::ElectricalLife => "Electrical Life",
            FactKind::ContactRatingVoltage => "Contact Rating Voltage",
            FactKind::ContactRatingCurrent => "Contact Rating Current",
            FactKind::CoilVoltage => "Coil Voltage",
            FactKind::CoilPower => "Coil Power",
            FactKind::ContactForm => "Contact Form",
            FactKind::ResolutionEncoder => "Resolution",
            FactKind::Airflow => "Airflow",
            FactKind::StaticPressure => "Static Pressure",
            FactKind::Torque => "Torque",
            FactKind::Speed => "Speed",
            FactKind::Other => "Other",
        }
    }

    /// Get the expected Unit for a FactKind (for validation).
    pub fn expected_unit(kind: FactKind) -> Option<Unit> {
        use FactKind::*;
        match kind {
            Voltage
            | ForwardVoltage
            | ReverseVoltage
            | ZenerVoltage
            | ClampingVoltage
            | BreakdownVoltage
            | ReverseWorkingVoltage
            | DropoutVoltage
            | InputOffsetVoltage
            | InputVoltageHigh
            | InputVoltageLow
            | OutputVoltageHigh
            | OutputVoltageLow
            | WorkingVoltage
            | BreakdownVoltageProtection
            | ClampingVoltageProtection
            | VoltageRating
            | SupplyVoltageMin
            | SupplyVoltageMax
            | VoltageLevelsMin
            | VoltageLevelsMax
            | GateThresholdVoltage => Some(Unit::Volt),
            Current
            | LeakageCurrent
            | InputBiasCurrent
            | QuiescentCurrent
            | OutputCurrent
            | ShutdownCurrent
            | CurrentLimit
            | RippleCurrentRating
            | SaturationCurrent
            | RmsCurrent
            | DriveLevel
            | ReadCurrent
            | WriteCurrent
            | StandbyCurrent
            | CurrentConsumptionActive
            | CurrentConsumptionSleep
            | CurrentConsumptionShutdown
            | CurrentTx
            | CurrentRx
            | CurrentSleep
            | CurrentPerContact
            | ContactRatingCurrent
            | CoilPower => Some(Unit::Ampere),
            Resistance | RdsOn | ZenerImpedance | Esr | Dcr => Some(Unit::Ohm),
            Capacitance
            | JunctionCapacitance
            | LoadCapacitance
            | CapacitanceIss
            | CapacitanceOss
            | CapacitanceRss
            | CapacitanceProtection => Some(Unit::Farad),
            Inductance => Some(Unit::Henry),
            Power | HeadphoneAmpPower | SpeakerAmpPower | PeakPulsePower => Some(Unit::Watt),
            Frequency | Bandwidth | SwitchingFrequency | MaxFrequency | MaxFrequencyMemory
            | MaxDataRate | ChannelBandwidth | MasterClock | Srf => Some(Unit::Hertz),
            Temperature | TemperatureMin | TemperatureMax | ThermalShutdown => {
                Some(Unit::DegreeCelsius)
            }
            Dimension | Pitch | ActuationTravel => Some(Unit::Millimetre),
            Gain | Cmrr | Psrr | Noise | FrequencyStability | Aging | LineRegulation
            | LoadRegulation | OutputNoise | Efficiency | AdditiveJitter | OutputSkew
            | DutyCycleDistortion | Accuracy | Sensitivity | NoiseDensity | Snr | ThdN
            | DynamicRange | Cmti | LinkBudget | LuminousIntensity | DominantWavelength
            | ViewingAngle | ColorTemperature | QFactor => Some(Unit::Unitless),
            GateCharge => Some(Unit::Farad), // Actually Coulomb, but using Farad as proxy
            TurnOnDelay
            | RiseTime
            | TurnOffDelay
            | FallTime
            | ReverseRecoveryTime
            | PropagationDelay
            | ResponseTime
            | ResponseTimeProtection
            | EraseTime
            | WriteTime
            | DataRetention => Some(Unit::Second),
            // Integer/count types use Unitless
            Mpn | Package | LifecycleStatus | Manufacturer | ConnectorType | MountingStyle
            | Orientation | LockingMechanism | ContactForm | DeviceType | OutputInterface
            | Protocol | ModulationSchemes | AntennaInterface | Interface | Organization
            | CoreFrequency | FlashSize | RamSize | GpioCount | AdcChannels | AdcResolution
            | DacChannels | DacResolution | TimerCount | UartCount | SpiCount | I2cCount
            | CanCount | Density | PageSize | SectorSize | BlockSize | EnduranceCycles
            | Channels | Positions | Rows | MatingCycles | DifferentialPairs | MechanicalLife
            | ElectricalLife | ChannelCount => Some(Unit::Unitless),
            Tolerance | SlewRate => Some(Unit::Unitless),
            // Missing variants that need explicit handling
            PeakPulseCurrent
            | Esl
            | OutputDriveCurrent
            | IsolationVoltage
            | EsdProtection
            | ModulationSchemes
            | RxSensitivity
            | TxPower
            | LinkBudget
            | Snr
            | ThdN
            | DynamicRange
            | ChannelCount
            | HeadphoneAmpPower
            | SpeakerAmpPower
            | MicBias
            | WorkingVoltage
            | BreakdownVoltageProtection
            | ClampingVoltageProtection
            | PeakPulseCurrentProtection
            | PeakPulsePower
            | HoldCurrent
            | TripCurrent
            | TripTime
            | ResponseTimeProtection
            | LeakageCurrentProtection
            | CapacitanceProtection
            | ConnectorType
            | Pitch
            | Positions
            | Rows
            | CurrentPerContact
            | VoltageRating
            | MatingCycles
            | MountingStyle
            | Orientation
            | LockingMechanism
            | Shielding
            | ImpedanceControlled
            | DifferentialPairs
            | DeviceType
            | ActuationForce
            | ActuationTravel
            | MechanicalLife
            | ElectricalLife
            | ContactRatingVoltage
            | ContactRatingCurrent
            | CoilVoltage
            | CoilPower
            | ContactForm
            | ResolutionEncoder
            | Airflow
            | StaticPressure
            | Torque
            | Speed => Some(Unit::Unitless),
            BufferSize | MeasurementRangeMin | MeasurementRangeMax | Resolution => {
                Some(Unit::Unitless)
            }
            Other => None,
        }
    }
}

/// Collection of facts for a single asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactCollection {
    pub asset_id: i64,
    pub facts: Vec<DatasheetFact>,
    pub extracted_at: String,
    pub parser_version: String,
}

impl FactCollection {
    pub fn new(asset_id: i64, parser_version: String) -> Self {
        Self {
            asset_id,
            facts: Vec::new(),
            extracted_at: chrono::Utc::now().to_rfc3339(),
            parser_version,
        }
    }

    pub fn add_fact(&mut self, fact: DatasheetFact) {
        self.facts.push(fact);
    }

    pub fn get_facts_by_kind(&self, kind: FactKind) -> Vec<&DatasheetFact> {
        self.facts.iter().filter(|f| f.kind == kind).collect()
    }

    pub fn get_verified_facts(&self) -> Vec<&DatasheetFact> {
        self.facts
            .iter()
            .filter(|f| f.provenance.verified)
            .collect()
    }

    pub fn get_unverified_facts(&self) -> Vec<&DatasheetFact> {
        self.facts
            .iter()
            .filter(|f| !f.provenance.verified)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eak_units::{PhysicalQuantity, Tolerance, Unit};

    #[test]
    fn test_parameter_fact_creation() {
        let pq = PhysicalQuantity::new(3.3, Unit::Volt);
        let fact = ParameterFact::new(pq);
        assert_eq!(fact.value.magnitude, 3.3);
        assert_eq!(fact.value.unit, Unit::Volt);
        assert!(fact.tolerance.is_none());
    }

    #[test]
    fn test_parameter_fact_with_tolerance() {
        let pq = PhysicalQuantity::new(10_000.0, Unit::Ohm);
        let tol = Tolerance::Relative(0.05); // 5%
        let fact = ParameterFact::with_tolerance(pq, tol);
        assert!(fact.tolerance.is_some());
    }

    #[test]
    fn test_parameter_fact_same_dimension() {
        let v1 = ParameterFact::new(PhysicalQuantity::new(3.3, Unit::Volt));
        let v2 = ParameterFact::new(PhysicalQuantity::new(5.0, Unit::Volt));
        let i1 = ParameterFact::new(PhysicalQuantity::new(10.0, Unit::Milliampere));

        assert!(v1.same_dimension(&v2));
        assert!(!v1.same_dimension(&i1));
    }

    #[test]
    fn test_parameter_fact_compare() {
        let v1 = ParameterFact::new(PhysicalQuantity::new(3.3, Unit::Volt));
        let v2 = ParameterFact::new(PhysicalQuantity::new(5.0, Unit::Volt));
        let v3 = ParameterFact::new(PhysicalQuantity::new(3300.0, Unit::Millivolt));

        assert_eq!(v1.compare(&v2).unwrap(), std::cmp::Ordering::Less);
        assert!(v1.same_value(&v3).unwrap());
    }

    #[test]
    fn test_fact_kind_description() {
        assert_eq!(
            DatasheetFact::kind_description(FactKind::Voltage),
            "Voltage"
        );
        assert_eq!(DatasheetFact::kind_description(FactKind::RdsOn), "RDS(on)");
        assert_eq!(
            DatasheetFact::kind_description(FactKind::Package),
            "Package"
        );
    }

    #[test]
    fn test_expected_unit() {
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Voltage),
            Some(Unit::Volt)
        );
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Current),
            Some(Unit::Ampere)
        );
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Resistance),
            Some(Unit::Ohm)
        );
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Capacitance),
            Some(Unit::Farad)
        );
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Frequency),
            Some(Unit::Hertz)
        );
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Temperature),
            Some(Unit::DegreeCelsius)
        );
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Package),
            Some(Unit::Unitless)
        );
        assert_eq!(
            DatasheetFact::expected_unit(FactKind::Mpn),
            Some(Unit::Unitless)
        );
    }

    #[test]
    fn test_fact_collection() {
        let mut coll = FactCollection::new(1, "1.0.0".to_string());
        let fact = DatasheetFact::new(
            1,
            FactKind::Voltage,
            ParameterFact::new(PhysicalQuantity::new(3.3, Unit::Volt)),
        );
        coll.add_fact(fact);

        assert_eq!(coll.facts.len(), 1);
        assert_eq!(coll.get_facts_by_kind(FactKind::Voltage).len(), 1);
        assert_eq!(coll.get_facts_by_kind(FactKind::Current).len(), 0);
    }
}
