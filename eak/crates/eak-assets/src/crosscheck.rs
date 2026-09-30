//! Metadata cross-check: compare extracted datasheet facts against component metadata.
//!
//! Detects matching values, conflicting values, missing metadata, and datasheet-only facts.

use crate::facts::{DatasheetFact, FactCollection, FactKind, ParameterFact, ProvenanceInfo};
use anyhow::Result;
use eak_domain::ComponentClass;
use eak_units::{PhysicalQuantity, UnitError};
use serde::{Deserialize, Serialize};

/// Simplified component metadata for cross-checking.
/// In a real implementation, this would come from the component database.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub core: ComponentCoreMetadata,
    pub family: Option<FamilyMetadata>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentCoreMetadata {
    pub operating_temperature_min: Option<PhysicalQuantity>,
    pub operating_temperature_max: Option<PhysicalQuantity>,
    pub max_operating_voltage: Option<PhysicalQuantity>,
    pub max_power_dissipation: Option<PhysicalQuantity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "family", content = "data")]
pub enum FamilyMetadata {
    Passive(PassiveMetadata),
    Diode(DiodeMetadata),
    Transistor(TransistorMetadata),
    AnalogIc(AnalogIcMetadata),
    PowerManagement(PowerManagementMetadata),
    DigitalLogic(DigitalLogicMetadata),
    Mcu(McuMetadata),
    Memory(MemoryMetadata),
    Communication(CommunicationMetadata),
    Sensor(SensorMetadata),
    RfWireless(RfWirelessMetadata),
    Audio(AudioMetadata),
    Protection(ProtectionMetadata),
    Connector(ConnectorMetadata),
    Electromechanical(ElectromechanicalMetadata),
    Specialized(SpecializedMetadata),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassiveMetadata {
    pub resistance: Option<PhysicalQuantity>,
    pub power_rating: Option<PhysicalQuantity>,
    pub max_working_voltage: Option<PhysicalQuantity>,
    pub temperature_coefficient: Option<PhysicalQuantity>,
    pub voltage_coefficient: Option<PhysicalQuantity>,
    pub capacitance: Option<PhysicalQuantity>,
    pub voltage_rating: Option<PhysicalQuantity>,
    pub esr: Option<PhysicalQuantity>,
    pub esl: Option<PhysicalQuantity>,
    pub dielectric_type: Option<String>,
    pub ripple_current_rating: Option<PhysicalQuantity>,
    pub inductance: Option<PhysicalQuantity>,
    pub dcr: Option<PhysicalQuantity>,
    pub saturation_current: Option<PhysicalQuantity>,
    pub rms_current: Option<PhysicalQuantity>,
    pub srf: Option<PhysicalQuantity>,
    pub q_factor: Option<PhysicalQuantity>,
    pub frequency: Option<PhysicalQuantity>,
    pub load_capacitance: Option<PhysicalQuantity>,
    pub drive_level: Option<PhysicalQuantity>,
    pub frequency_stability: Option<PhysicalQuantity>,
    pub aging: Option<PhysicalQuantity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiodeMetadata {
    pub forward_voltage: Option<PhysicalQuantity>,
    pub reverse_voltage: Option<PhysicalQuantity>,
    pub forward_current: Option<PhysicalQuantity>,
    pub reverse_recovery_time: Option<PhysicalQuantity>,
    pub junction_capacitance: Option<PhysicalQuantity>,
    pub leakage_current: Option<PhysicalQuantity>,
    pub power_dissipation: Option<PhysicalQuantity>,
    pub thermal_resistance_jc: Option<PhysicalQuantity>,
    pub thermal_resistance_ja: Option<PhysicalQuantity>,
    pub zener_voltage: Option<PhysicalQuantity>,
    pub zener_impedance: Option<PhysicalQuantity>,
    pub zener_temperature_coefficient: Option<PhysicalQuantity>,
    pub reverse_working_voltage: Option<PhysicalQuantity>,
    pub breakdown_voltage: Option<PhysicalQuantity>,
    pub peak_pulse_current: Option<PhysicalQuantity>,
    pub clamping_voltage: Option<PhysicalQuantity>,
    pub luminous_intensity: Option<PhysicalQuantity>,
    pub dominant_wavelength: Option<PhysicalQuantity>,
    pub viewing_angle: Option<PhysicalQuantity>,
    pub color_temperature: Option<PhysicalQuantity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransistorMetadata {
    pub drain_source_voltage: Option<PhysicalQuantity>,
    pub gate_source_voltage: Option<PhysicalQuantity>,
    pub continuous_drain_current: Option<PhysicalQuantity>,
    pub pulsed_drain_current: Option<PhysicalQuantity>,
    pub rds_on: Option<PhysicalQuantity>,
    pub gate_threshold_voltage: Option<PhysicalQuantity>,
    pub total_gate_charge: Option<PhysicalQuantity>,
    pub input_capacitance: Option<PhysicalQuantity>,
    pub output_capacitance: Option<PhysicalQuantity>,
    pub reverse_transfer_capacitance: Option<PhysicalQuantity>,
    pub turn_on_delay: Option<PhysicalQuantity>,
    pub rise_time: Option<PhysicalQuantity>,
    pub turn_off_delay: Option<PhysicalQuantity>,
    pub fall_time: Option<PhysicalQuantity>,
    pub collector_emitter_voltage: Option<PhysicalQuantity>,
    pub collector_current: Option<PhysicalQuantity>,
    pub dc_current_gain: Option<PhysicalQuantity>,
    pub collector_emitter_sat_voltage: Option<PhysicalQuantity>,
    pub transition_frequency: Option<PhysicalQuantity>,
    pub power_dissipation: Option<PhysicalQuantity>,
    pub collector_emitter_voltage_ces: Option<PhysicalQuantity>,
    pub gate_emitter_threshold: Option<PhysicalQuantity>,
    pub collector_emitter_sat: Option<PhysicalQuantity>,
    pub turn_on_energy: Option<PhysicalQuantity>,
    pub turn_off_energy: Option<PhysicalQuantity>,
    pub reverse_recovery_charge: Option<PhysicalQuantity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalogIcMetadata {
    pub gain_bandwidth_product: Option<PhysicalQuantity>,
    pub slew_rate: Option<PhysicalQuantity>,
    pub input_offset_voltage: Option<PhysicalQuantity>,
    pub input_offset_voltage_drift: Option<PhysicalQuantity>,
    pub input_bias_current: Option<PhysicalQuantity>,
    pub input_offset_current: Option<PhysicalQuantity>,
    pub input_voltage_noise: Option<PhysicalQuantity>,
    pub input_current_noise: Option<PhysicalQuantity>,
    pub cmrr: Option<PhysicalQuantity>,
    pub psrr: Option<PhysicalQuantity>,
    pub supply_voltage_range_min: Option<PhysicalQuantity>,
    pub supply_voltage_range_max: Option<PhysicalQuantity>,
    pub quiescent_current: Option<PhysicalQuantity>,
    pub output_current: Option<PhysicalQuantity>,
    pub rail_to_rail_input: Option<bool>,
    pub rail_to_rail_output: Option<bool>,
    pub propagation_delay: Option<PhysicalQuantity>,
    pub overdrive_voltage: Option<PhysicalQuantity>,
    pub hysteresis: Option<PhysicalQuantity>,
    pub output_type: Option<String>,
    pub initial_accuracy: Option<PhysicalQuantity>,
    pub vref_temperature_coefficient: Option<PhysicalQuantity>,
    pub noise_0_1_10hz: Option<PhysicalQuantity>,
    pub noise_10hz_10khz: Option<PhysicalQuantity>,
    pub load_regulation: Option<PhysicalQuantity>,
    pub line_regulation: Option<PhysicalQuantity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerManagementMetadata {
    pub input_voltage_range_min: Option<PhysicalQuantity>,
    pub input_voltage_range_max: Option<PhysicalQuantity>,
    pub output_voltage_range_min: Option<PhysicalQuantity>,
    pub output_voltage_range_max: Option<PhysicalQuantity>,
    pub output_current: Option<PhysicalQuantity>,
    pub switching_frequency: Option<PhysicalQuantity>,
    pub efficiency: Option<PhysicalQuantity>,
    pub control_mode: Option<String>,
    pub enable_pin: Option<bool>,
    pub sync_pin: Option<bool>,
    pub power_good: Option<bool>,
    pub soft_start: Option<PhysicalQuantity>,
    pub dropout_voltage: Option<PhysicalQuantity>,
    pub ldo_line_regulation: Option<PhysicalQuantity>,
    pub ldo_load_regulation: Option<PhysicalQuantity>,
    pub output_noise: Option<PhysicalQuantity>,
    pub psrr: Option<PhysicalQuantity>,
    pub quiescent_current: Option<PhysicalQuantity>,
    pub shutdown_current: Option<PhysicalQuantity>,
    pub current_limit: Option<PhysicalQuantity>,
    pub thermal_shutdown: Option<PhysicalQuantity>,
    pub battery_chemistry: Option<String>,
    pub charge_voltage: Option<PhysicalQuantity>,
    pub charge_current: Option<PhysicalQuantity>,
    pub termination_current: Option<PhysicalQuantity>,
    pub precharge_current: Option<PhysicalQuantity>,
    pub safety_timer: Option<PhysicalQuantity>,
    pub fuel_gauge_algorithm: Option<String>,
    pub cell_count: Option<u32>,
    pub balancing_current: Option<PhysicalQuantity>,
    pub current_limit_accuracy: Option<PhysicalQuantity>,
    pub response_time: Option<PhysicalQuantity>,
    pub auto_retry: Option<bool>,
    pub fault_reporting: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigitalLogicMetadata {
    pub propagation_delay: Option<PhysicalQuantity>,
    pub output_drive_current: Option<PhysicalQuantity>,
    pub input_voltage_high: Option<PhysicalQuantity>,
    pub input_voltage_low: Option<PhysicalQuantity>,
    pub output_voltage_high: Option<PhysicalQuantity>,
    pub output_voltage_low: Option<PhysicalQuantity>,
    pub supply_voltage_range_min: Option<PhysicalQuantity>,
    pub supply_voltage_range_max: Option<PhysicalQuantity>,
    pub quiescent_current: Option<PhysicalQuantity>,
    pub input_capacitance: Option<PhysicalQuantity>,
    pub max_frequency: Option<PhysicalQuantity>,
    pub voltage_range_a_min: Option<PhysicalQuantity>,
    pub voltage_range_a_max: Option<PhysicalQuantity>,
    pub voltage_range_b_min: Option<PhysicalQuantity>,
    pub voltage_range_b_max: Option<PhysicalQuantity>,
    pub direction_control: Option<String>,
    pub additive_jitter: Option<PhysicalQuantity>,
    pub output_skew: Option<PhysicalQuantity>,
    pub duty_cycle_distortion: Option<PhysicalQuantity>,
    pub output_format: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McuMetadata {
    pub core_architecture: Option<String>,
    pub core_frequency: Option<PhysicalQuantity>,
    pub flash_size: Option<PhysicalQuantity>,
    pub ram_size: Option<PhysicalQuantity>,
    pub eeprom_size: Option<PhysicalQuantity>,
    pub gpio_count: Option<u32>,
    pub adc_channels: Option<u32>,
    pub adc_resolution: Option<u32>,
    pub dac_channels: Option<u32>,
    pub dac_resolution: Option<u32>,
    pub timer_16bit_count: Option<u32>,
    pub timer_32bit_count: Option<u32>,
    pub advanced_control_timer_count: Option<u32>,
    pub basic_timer_count: Option<u32>,
    pub lptim_count: Option<u32>,
    pub uart_count: Option<u32>,
    pub spi_count: Option<u32>,
    pub i2c_count: Option<u32>,
    pub i3c_count: Option<u32>,
    pub can_count: Option<u32>,
    pub can_fd: Option<bool>,
    pub usb_type: Option<String>,
    pub ethernet: Option<bool>,
    pub sdio: Option<bool>,
    pub trustzone: Option<bool>,
    pub mpu: Option<bool>,
    pub secure_boot: Option<bool>,
    pub hardware_crypto: Vec<String>,
    pub tamper_detection: Option<bool>,
    pub package_options: Vec<String>,
    pub temperature_grade: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub density: Option<PhysicalQuantity>,
    pub organization: Option<String>,
    pub interface: Option<String>,
    pub max_frequency: Option<PhysicalQuantity>,
    pub supply_voltage_min: Option<PhysicalQuantity>,
    pub supply_voltage_max: Option<PhysicalQuantity>,
    pub read_current: Option<PhysicalQuantity>,
    pub write_current: Option<PhysicalQuantity>,
    pub standby_current: Option<PhysicalQuantity>,
    pub page_size: Option<u32>,
    pub sector_size: Option<u32>,
    pub block_size: Option<u32>,
    pub erase_time: Option<PhysicalQuantity>,
    pub write_time: Option<PhysicalQuantity>,
    pub endurance_cycles: Option<u32>,
    pub data_retention: Option<PhysicalQuantity>,
    pub hardware_ecc: Option<bool>,
    pub write_protect: Option<String>,
    pub unique_id: Option<bool>,
    pub security_features: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationMetadata {
    pub protocol: Option<String>,
    pub max_data_rate: Option<PhysicalQuantity>,
    pub voltage_levels_min: Option<PhysicalQuantity>,
    pub voltage_levels_max: Option<PhysicalQuantity>,
    pub channels: Option<u32>,
    pub isolation_voltage: Option<PhysicalQuantity>,
    pub cmti: Option<PhysicalQuantity>,
    pub esd_protection: Option<PhysicalQuantity>,
    pub transceiver_integrated: Option<bool>,
    pub phy_integrated: Option<bool>,
    pub mac_integrated: Option<bool>,
    pub buffer_size: Option<PhysicalQuantity>,
    pub dma_support: Option<bool>,
    pub wakeup_capability: Option<bool>,
    pub automotive_qualified: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensorMetadata {
    pub sensor_type: Option<String>,
    pub measurement_range_min: Option<PhysicalQuantity>,
    pub measurement_range_max: Option<PhysicalQuantity>,
    pub accuracy: Option<PhysicalQuantity>,
    pub resolution: Option<PhysicalQuantity>,
    pub sensitivity: Option<PhysicalQuantity>,
    pub noise_density: Option<PhysicalQuantity>,
    pub bandwidth: Option<PhysicalQuantity>,
    pub output_interface: Option<String>,
    pub supply_voltage_min: Option<PhysicalQuantity>,
    pub supply_voltage_max: Option<PhysicalQuantity>,
    pub current_consumption_active: Option<PhysicalQuantity>,
    pub current_consumption_sleep: Option<PhysicalQuantity>,
    pub current_consumption_shutdown: Option<PhysicalQuantity>,
    pub temperature_range_min: Option<PhysicalQuantity>,
    pub temperature_range_max: Option<PhysicalQuantity>,
    pub response_time: Option<PhysicalQuantity>,
    pub calibration: Option<String>,
    pub fiducial_reference: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RfWirelessMetadata {
    pub frequency_range_min: Option<PhysicalQuantity>,
    pub frequency_range_max: Option<PhysicalQuantity>,
    pub tx_power: Option<PhysicalQuantity>,
    pub rx_sensitivity: Option<PhysicalQuantity>,
    pub modulation_schemes: Vec<String>,
    pub channel_bandwidth: Option<PhysicalQuantity>,
    pub antenna_interface: Option<String>,
    pub pa_integrated: Option<bool>,
    pub lna_integrated: Option<bool>,
    pub coexistence_support: Option<bool>,
    pub regulatory_certifications: Vec<String>,
    pub current_tx: Option<PhysicalQuantity>,
    pub current_rx: Option<PhysicalQuantity>,
    pub current_sleep: Option<PhysicalQuantity>,
    pub link_budget: Option<PhysicalQuantity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub resolution_bits: Option<u32>,
    pub sample_rates: Vec<PhysicalQuantity>,
    pub snr: Option<PhysicalQuantity>,
    pub thd_n: Option<PhysicalQuantity>,
    pub dynamic_range: Option<PhysicalQuantity>,
    pub channel_count: Option<u32>,
    pub interface: Option<String>,
    pub master_clock: Option<PhysicalQuantity>,
    pub pll_integrated: Option<bool>,
    pub headphone_amp_power: Option<PhysicalQuantity>,
    pub speaker_amp_power: Option<PhysicalQuantity>,
    pub mic_bias: Option<PhysicalQuantity>,
    pub dsp_features: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtectionMetadata {
    pub protection_type: Option<String>,
    pub working_voltage: Option<PhysicalQuantity>,
    pub breakdown_voltage: Option<PhysicalQuantity>,
    pub clamping_voltage: Option<PhysicalQuantity>,
    pub peak_pulse_current: Option<PhysicalQuantity>,
    pub peak_pulse_power: Option<PhysicalQuantity>,
    pub hold_current: Option<PhysicalQuantity>,
    pub trip_current: Option<PhysicalQuantity>,
    pub trip_time: Option<PhysicalQuantity>,
    pub reset_type: Option<String>,
    pub response_time: Option<PhysicalQuantity>,
    pub leakage_current: Option<PhysicalQuantity>,
    pub capacitance: Option<PhysicalQuantity>,
    pub isolation_voltage: Option<PhysicalQuantity>,
    pub bidirectional: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectorMetadata {
    pub connector_type: Option<String>,
    pub pitch: Option<PhysicalQuantity>,
    pub positions: Option<u32>,
    pub rows: Option<u32>,
    pub current_per_contact: Option<PhysicalQuantity>,
    pub voltage_rating: Option<PhysicalQuantity>,
    pub mating_cycles: Option<u32>,
    pub mounting_style: Option<String>,
    pub orientation: Option<String>,
    pub locking_mechanism: Option<String>,
    pub shielding: Option<bool>,
    pub impedance_controlled: Option<bool>,
    pub differential_pairs: Option<u32>,
    pub operating_temperature_min: Option<PhysicalQuantity>,
    pub operating_temperature_max: Option<PhysicalQuantity>,
    pub flange_mount: Option<bool>,
    pub panel_mount: Option<bool>,
    pub cable_accommodation: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElectromechanicalMetadata {
    pub device_type: Option<String>,
    pub actuation_force: Option<PhysicalQuantity>,
    pub actuation_travel: Option<PhysicalQuantity>,
    pub mechanical_life: Option<u32>,
    pub electrical_life: Option<u32>,
    pub contact_rating_voltage: Option<PhysicalQuantity>,
    pub contact_rating_current: Option<PhysicalQuantity>,
    pub coil_voltage: Option<PhysicalQuantity>,
    pub coil_power: Option<PhysicalQuantity>,
    pub contact_form: Option<String>,
    pub resolution: Option<PhysicalQuantity>,
    pub detent: Option<bool>,
    pub airflow: Option<PhysicalQuantity>,
    pub static_pressure: Option<PhysicalQuantity>,
    pub torque: Option<PhysicalQuantity>,
    pub speed: Option<PhysicalQuantity>,
    pub feedback_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecializedMetadata {
    pub application_domain: Option<String>,
    pub qualification_standard: Option<String>,
    pub radiation_tolerance: Option<PhysicalQuantity>,
    pub operating_temperature_extreme_min: Option<PhysicalQuantity>,
    pub operating_temperature_extreme_max: Option<PhysicalQuantity>,
    pub hermetic_sealing: Option<bool>,
    pub biocompatibility: Option<String>,
    pub vacuum_compatible: Option<bool>,
    pub magnetic_field_immunity: Option<PhysicalQuantity>,
    pub special_requirements: Vec<String>,
}

impl FamilyMetadata {
    pub fn component_class(&self) -> ComponentClass {
        match self {
            FamilyMetadata::Passive(_) => ComponentClass::Resistor, // Map to Resistor for passives
            FamilyMetadata::Diode(_) => ComponentClass::Ic,         // Map to IC for diodes
            FamilyMetadata::Transistor(_) => ComponentClass::Ic,    // Map to IC for transistors
            FamilyMetadata::AnalogIc(_) => ComponentClass::Ic,
            FamilyMetadata::PowerManagement(_) => ComponentClass::Regulator,
            FamilyMetadata::DigitalLogic(_) => ComponentClass::Ic,
            FamilyMetadata::Mcu(_) => ComponentClass::Ic,
            FamilyMetadata::Memory(_) => ComponentClass::Ic,
            FamilyMetadata::Communication(_) => ComponentClass::Ic,
            FamilyMetadata::Sensor(_) => ComponentClass::Ic,
            FamilyMetadata::RfWireless(_) => ComponentClass::Ic,
            FamilyMetadata::Audio(_) => ComponentClass::Ic,
            FamilyMetadata::Protection(_) => ComponentClass::Ic,
            FamilyMetadata::Connector(_) => ComponentClass::Connector,
            FamilyMetadata::Electromechanical(_) => ComponentClass::Ic,
            FamilyMetadata::Specialized(_) => ComponentClass::Ic,
        }
    }

    pub fn default_for(class: ComponentClass) -> Self {
        match class {
            ComponentClass::Resistor | ComponentClass::Capacitor => {
                FamilyMetadata::Passive(PassiveMetadata::default())
            }
            ComponentClass::Connector => FamilyMetadata::Connector(ConnectorMetadata::default()),
            ComponentClass::Regulator => {
                FamilyMetadata::PowerManagement(PowerManagementMetadata::default())
            }
            ComponentClass::Ic => FamilyMetadata::AnalogIc(AnalogIcMetadata::default()),
            _ => FamilyMetadata::Passive(PassiveMetadata::default()),
        }
    }
}

/// Result of comparing a single metadata field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldComparisonResult {
    /// Fact kind being compared
    pub fact_kind: FactKind,
    /// Human-readable name
    pub name: String,
    /// Value from component metadata (if present)
    pub metadata_value: Option<ParameterFact>,
    /// Value from datasheet (if present)
    pub datasheet_value: Option<ParameterFact>,
    /// Comparison result
    pub comparison: ComparisonType,
    /// Confidence in the comparison (0.0 - 1.0)
    pub confidence: f64,
    /// Human-readable details
    pub details: String,
}

/// Type of comparison result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonType {
    /// Values match within tolerance
    Match,
    /// Values conflict (different values for same parameter)
    Conflict,
    /// Metadata has value, datasheet doesn't
    MissingInDatasheet,
    /// Datasheet has value, metadata doesn't
    DatasheetOnly,
    /// Neither has a value
    BothMissing,
    /// Cannot compare (dimension mismatch, parse error, etc.)
    Incomparable,
}

impl std::fmt::Display for ComparisonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonType::Match => write!(f, "MATCH"),
            ComparisonType::Conflict => write!(f, "CONFLICT"),
            ComparisonType::MissingInDatasheet => write!(f, "MISSING_IN_DATASHEET"),
            ComparisonType::DatasheetOnly => write!(f, "DATASHEET_ONLY"),
            ComparisonType::BothMissing => write!(f, "BOTH_MISSING"),
            ComparisonType::Incomparable => write!(f, "INCOMPARABLE"),
        }
    }
}

/// Complete cross-check report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCheckReport {
    /// Component MPN
    pub component_mpn: String,
    /// Component manufacturer
    pub component_manufacturer: String,
    /// Component class
    pub component_class: ComponentClass,
    /// Individual field comparisons
    pub field_comparisons: Vec<FieldComparisonResult>,
    /// Summary counts
    pub summary: CrossCheckSummary,
    /// Overall assessment
    pub assessment: CrossCheckAssessment,
    /// Timestamp
    pub checked_at: String,
}

/// Summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCheckSummary {
    pub total_fields_checked: usize,
    pub matches: usize,
    pub conflicts: usize,
    pub missing_in_datasheet: usize,
    pub datasheet_only: usize,
    pub both_missing: usize,
    pub incomparable: usize,
}

/// Overall assessment of the cross-check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrossCheckAssessment {
    /// All checked fields match, no conflicts
    Clean,
    /// Some fields missing in datasheet but no conflicts
    Incomplete,
    /// One or more conflicts detected
    HasConflicts,
    /// Significant mismatches, identity may be wrong
    Suspicious,
}

impl std::fmt::Display for CrossCheckAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrossCheckAssessment::Clean => write!(f, "CLEAN"),
            CrossCheckAssessment::Incomplete => write!(f, "INCOMPLETE"),
            CrossCheckAssessment::HasConflicts => write!(f, "HAS_CONFLICTS"),
            CrossCheckAssessment::Suspicious => write!(f, "SUSPICIOUS"),
        }
    }
}

/// Map FactKind to metadata field accessor for different component classes.
type MetadataAccessor = fn(&ComponentMetadata) -> Option<ParameterFact>;

/// Get the metadata accessor for a given FactKind and ComponentClass.
fn get_metadata_accessor(kind: FactKind, class: ComponentClass) -> Option<MetadataAccessor> {
    use ComponentClass::*;
    use FactKind::*;

    // Core metadata (all classes)
    let accessor: Option<MetadataAccessor> = match kind {
        TemperatureMin => Some(|m: &ComponentMetadata| -> Option<ParameterFact> {
            m.core
                .operating_temperature_min
                .map(|v| ParameterFact::new(v))
        }),
        TemperatureMax => Some(|m: &ComponentMetadata| -> Option<ParameterFact> {
            m.core
                .operating_temperature_max
                .map(|v| ParameterFact::new(v))
        }),
        Voltage => Some(|m: &ComponentMetadata| -> Option<ParameterFact> {
            m.core.max_operating_voltage.map(|v| ParameterFact::new(v))
        }),
        Power => Some(|m: &ComponentMetadata| -> Option<ParameterFact> {
            m.core.max_power_dissipation.map(|v| ParameterFact::new(v))
        }),
        _ => None,
    };

    accessor.or_else(|| {
        // Family-specific metadata
        match class {
            Resistor | Capacitor => get_passive_accessor(kind),
            Connector => get_connector_accessor(kind),
            Regulator => get_power_mgmt_accessor(kind),
            Ic => get_analog_ic_accessor(kind),
        }
    })
}

// Family-specific accessors
fn get_passive_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Resistance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p.resistance.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Power => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p.power_rating.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p
                    .max_working_voltage
                    .or(p.voltage_rating)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Tolerance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => {
                    p.temperature_coefficient.map(|v| ParameterFact::new(v))
                }
                _ => None,
            })
        }),
        Capacitance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p.capacitance.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Resistance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p.esr.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Inductance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p.inductance.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Resistance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p.dcr.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p
                    .saturation_current
                    .or(p.rms_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Passive(p) => p.srf.or(p.frequency).map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_diode_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Diode(d) => d
                    .forward_voltage
                    .or(d.reverse_voltage)
                    .or(d.zener_voltage)
                    .or(d.reverse_working_voltage)
                    .or(d.breakdown_voltage)
                    .or(d.clamping_voltage)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Diode(d) => d
                    .forward_current
                    .or(d.leakage_current)
                    .or(d.peak_pulse_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Time => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Diode(d) => d.reverse_recovery_time.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Capacitance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Diode(d) => d.junction_capacitance.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Power => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Diode(d) => d.power_dissipation.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_transistor_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Transistor(t) => t
                    .drain_source_voltage
                    .or(t.gate_source_voltage)
                    .or(t.collector_emitter_voltage)
                    .or(t.collector_emitter_voltage_ces)
                    .or(t.collector_emitter_sat_voltage)
                    .or(t.collector_emitter_sat)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Transistor(t) => t
                    .continuous_drain_current
                    .or(t.collector_current)
                    .or(t.pulsed_drain_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Resistance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Transistor(t) => t
                    .rds_on
                    .or(t.collector_emitter_sat_voltage)
                    .or(t.collector_emitter_sat)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Transistor(t) => {
                    t.gate_threshold_voltage.map(|v| ParameterFact::new(v))
                }
                _ => None,
            })
        }),
        Capacitance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Transistor(t) => t
                    .input_capacitance
                    .or(t.output_capacitance)
                    .or(t.reverse_transfer_capacitance)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Power => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Transistor(t) => t.power_dissipation.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Time => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Transistor(t) => t
                    .turn_on_delay
                    .or(t.rise_time)
                    .or(t.turn_off_delay)
                    .or(t.fall_time)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_analog_ic_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::AnalogIc(a) => {
                    a.gain_bandwidth_product.map(|v| ParameterFact::new(v))
                }
                _ => None,
            })
        }),
        SlewRate => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::AnalogIc(a) => a.slew_rate.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::AnalogIc(a) => a
                    .input_offset_voltage
                    .or(a.supply_voltage_range_min)
                    .or(a.supply_voltage_range_max)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::AnalogIc(a) => a
                    .input_bias_current
                    .or(a.quiescent_current)
                    .or(a.output_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Noise => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::AnalogIc(a) => a.input_voltage_noise.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_power_mgmt_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::PowerManagement(p) => p
                    .input_voltage_range_min
                    .or(p.input_voltage_range_max)
                    .or(p.output_voltage_range_min)
                    .or(p.output_voltage_range_max)
                    .or(p.dropout_voltage)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::PowerManagement(p) => p
                    .output_current
                    .or(p.quiescent_current)
                    .or(p.shutdown_current)
                    .or(p.current_limit)
                    .or(p.balancing_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::PowerManagement(p) => {
                    p.switching_frequency.map(|v| ParameterFact::new(v))
                }
                _ => None,
            })
        }),
        Efficiency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::PowerManagement(p) => p.efficiency.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Temperature => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::PowerManagement(p) => {
                    p.thermal_shutdown.map(|v| ParameterFact::new(v))
                }
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_digital_logic_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Time => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::DigitalLogic(d) => {
                    d.propagation_delay.map(|v| ParameterFact::new(v))
                }
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::DigitalLogic(d) => d
                    .output_drive_current
                    .or(d.quiescent_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::DigitalLogic(d) => d
                    .input_voltage_high
                    .or(d.input_voltage_low)
                    .or(d.output_voltage_high)
                    .or(d.output_voltage_low)
                    .or(d.supply_voltage_range_min)
                    .or(d.supply_voltage_range_max)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::DigitalLogic(d) => d.max_frequency.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_mcu_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Mcu(m) => m.core_frequency.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_memory_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Memory(m) => m.max_frequency.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Memory(m) => m
                    .supply_voltage_min
                    .or(m.supply_voltage_max)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Memory(m) => m
                    .read_current
                    .or(m.write_current)
                    .or(m.standby_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Time => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Memory(m) => m
                    .erase_time
                    .or(m.write_time)
                    .or(m.data_retention)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_communication_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Communication(c) => c.max_data_rate.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Communication(c) => c
                    .voltage_levels_min
                    .or(c.voltage_levels_max)
                    .or(c.isolation_voltage)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_sensor_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Sensor(s) => s
                    .supply_voltage_min
                    .or(s.supply_voltage_max)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Sensor(s) => s
                    .current_consumption_active
                    .or(s.current_consumption_sleep)
                    .or(s.current_consumption_shutdown)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Sensor(s) => s.bandwidth.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Temperature => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Sensor(s) => s
                    .temperature_range_min
                    .or(s.temperature_range_max)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Time => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Sensor(s) => s.response_time.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_rf_wireless_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::RfWireless(r) => r
                    .frequency_range_min
                    .or(r.frequency_range_max)
                    .or(r.channel_bandwidth)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Power => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::RfWireless(r) => r.tx_power.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::RfWireless(r) => r.rx_sensitivity.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::RfWireless(r) => r
                    .current_tx
                    .or(r.current_rx)
                    .or(r.current_sleep)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_audio_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Frequency => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Audio(a) => a
                    .sample_rates
                    .first()
                    .cloned()
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Power => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Audio(a) => a
                    .headphone_amp_power
                    .or(a.speaker_amp_power)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Audio(a) => a.mic_bias.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_protection_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Protection(p) => p
                    .working_voltage
                    .or(p.breakdown_voltage)
                    .or(p.clamping_voltage)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Protection(p) => p
                    .peak_pulse_current
                    .or(p.hold_current)
                    .or(p.trip_current)
                    .or(p.leakage_current)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Power => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Protection(p) => p.peak_pulse_power.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Time => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Protection(p) => p
                    .trip_time
                    .or(p.response_time)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Capacitance => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Protection(p) => p.capacitance.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_connector_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Dimension => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Connector(c) => c.pitch.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Connector(c) => {
                    c.current_per_contact.map(|v| ParameterFact::new(v))
                }
                _ => None,
            })
        }),
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Connector(c) => c.voltage_rating.map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_electromechanical_accessor(kind: FactKind) -> Option<MetadataAccessor> {
    use FactKind::*;
    match kind {
        Voltage => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Electromechanical(e) => e
                    .contact_rating_voltage
                    .or(e.coil_voltage)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        Current => Some(|m| {
            m.family.as_ref().and_then(|f| match f {
                FamilyMetadata::Electromechanical(e) => e
                    .contact_rating_current
                    .or(e.coil_power)
                    .map(|v| ParameterFact::new(v)),
                _ => None,
            })
        }),
        _ => None,
    }
}

fn get_specialized_accessor(_kind: FactKind) -> Option<MetadataAccessor> {
    None
}

/// Compare two ParameterFacts for equality (within tolerance).
fn compare_facts(meta: &ParameterFact, ds: &ParameterFact) -> Result<ComparisonType, UnitError> {
    // Check dimension compatibility
    if meta.value.dimension() != ds.value.dimension() {
        return Ok(ComparisonType::Incomparable);
    }

    // Use same_value which handles tolerance
    match meta.same_value(ds) {
        Ok(true) => Ok(ComparisonType::Match),
        Ok(false) => Ok(ComparisonType::Conflict),
        Err(_) => Ok(ComparisonType::Incomparable),
    }
}

/// Perform cross-check between component metadata and datasheet facts.
pub fn cross_check_metadata(
    component_mpn: &str,
    component_manufacturer: &str,
    component_class: ComponentClass,
    metadata: &ComponentMetadata,
    facts: &FactCollection,
) -> CrossCheckReport {
    let mut field_comparisons = Vec::new();

    // Get all fact kinds present in either metadata or datasheet
    let mut checked_kinds = std::collections::HashSet::new();

    // Check metadata fields that have accessors
    let all_kinds = [
        FactKind::Voltage,
        FactKind::Current,
        FactKind::Resistance,
        FactKind::Capacitance,
        FactKind::Inductance,
        FactKind::Power,
        FactKind::Frequency,
        FactKind::Temperature,
        FactKind::TemperatureMin,
        FactKind::TemperatureMax,
        FactKind::Tolerance,
        FactKind::Gain,
        FactKind::Bandwidth,
        FactKind::SlewRate,
        FactKind::Noise,
        FactKind::Cmrr,
        FactKind::Psrr,
        FactKind::InputOffsetVoltage,
        FactKind::InputBiasCurrent,
        FactKind::QuiescentCurrent,
        FactKind::OutputCurrent,
        FactKind::RdsOn,
        FactKind::GateThresholdVoltage,
        FactKind::GateCharge,
        FactKind::CapacitanceIss,
        FactKind::CapacitanceOss,
        FactKind::CapacitanceRss,
        FactKind::TurnOnDelay,
        FactKind::RiseTime,
        FactKind::TurnOffDelay,
        FactKind::FallTime,
        FactKind::ReverseRecoveryTime,
        FactKind::JunctionCapacitance,
        FactKind::LeakageCurrent,
        FactKind::ForwardVoltage,
        FactKind::ReverseVoltage,
        FactKind::ZenerVoltage,
        FactKind::ZenerImpedance,
        FactKind::ClampingVoltage,
        FactKind::PeakPulseCurrent,
        FactKind::BreakdownVoltage,
        FactKind::ReverseWorkingVoltage,
        FactKind::LuminousIntensity,
        FactKind::DominantWavelength,
        FactKind::ViewingAngle,
        FactKind::ColorTemperature,
        FactKind::Esr,
        FactKind::Esl,
        FactKind::RippleCurrentRating,
        FactKind::Dcr,
        FactKind::SaturationCurrent,
        FactKind::RmsCurrent,
        FactKind::Srf,
        FactKind::QFactor,
        FactKind::LoadCapacitance,
        FactKind::DriveLevel,
        FactKind::FrequencyStability,
        FactKind::Aging,
        FactKind::DropoutVoltage,
        FactKind::LineRegulation,
        FactKind::LoadRegulation,
        FactKind::OutputNoise,
        FactKind::ShutdownCurrent,
        FactKind::CurrentLimit,
        FactKind::ThermalShutdown,
        FactKind::Efficiency,
        FactKind::SwitchingFrequency,
        FactKind::PropagationDelay,
        FactKind::OutputDriveCurrent,
        FactKind::InputVoltageHigh,
        FactKind::InputVoltageLow,
        FactKind::OutputVoltageHigh,
        FactKind::OutputVoltageLow,
        FactKind::MaxFrequency,
        FactKind::AdditiveJitter,
        FactKind::OutputSkew,
        FactKind::DutyCycleDistortion,
        FactKind::CoreFrequency,
        FactKind::Density,
        FactKind::Organization,
        FactKind::Interface,
        FactKind::MaxFrequencyMemory,
        FactKind::SupplyVoltageMin,
        FactKind::SupplyVoltageMax,
        FactKind::ReadCurrent,
        FactKind::WriteCurrent,
        FactKind::StandbyCurrent,
        FactKind::PageSize,
        FactKind::SectorSize,
        FactKind::BlockSize,
        FactKind::EraseTime,
        FactKind::WriteTime,
        FactKind::EnduranceCycles,
        FactKind::DataRetention,
        FactKind::Protocol,
        FactKind::MaxDataRate,
        FactKind::VoltageLevelsMin,
        FactKind::VoltageLevelsMax,
        FactKind::Channels,
        FactKind::IsolationVoltage,
        FactKind::Cmti,
        FactKind::EsdProtection,
        FactKind::BufferSize,
        FactKind::MeasurementRangeMin,
        FactKind::MeasurementRangeMax,
        FactKind::Accuracy,
        FactKind::Resolution,
        FactKind::Sensitivity,
        FactKind::NoiseDensity,
        FactKind::Bandwidth,
        FactKind::OutputInterface,
        FactKind::CurrentConsumptionActive,
        FactKind::CurrentConsumptionSleep,
        FactKind::CurrentConsumptionShutdown,
        FactKind::ResponseTime,
        FactKind::TxPower,
        FactKind::RxSensitivity,
        FactKind::ModulationSchemes,
        FactKind::ChannelBandwidth,
        FactKind::AntennaInterface,
        FactKind::CurrentTx,
        FactKind::CurrentRx,
        FactKind::CurrentSleep,
        FactKind::LinkBudget,
        FactKind::Snr,
        FactKind::ThdN,
        FactKind::DynamicRange,
        FactKind::ChannelCount,
        FactKind::MasterClock,
        FactKind::HeadphoneAmpPower,
        FactKind::SpeakerAmpPower,
        FactKind::MicBias,
        FactKind::WorkingVoltage,
        FactKind::BreakdownVoltageProtection,
        FactKind::ClampingVoltageProtection,
        FactKind::PeakPulseCurrentProtection,
        FactKind::PeakPulsePower,
        FactKind::HoldCurrent,
        FactKind::TripCurrent,
        FactKind::TripTime,
        FactKind::ResponseTimeProtection,
        FactKind::LeakageCurrentProtection,
        FactKind::CapacitanceProtection,
        FactKind::ConnectorType,
        FactKind::Pitch,
        FactKind::Positions,
        FactKind::Rows,
        FactKind::CurrentPerContact,
        FactKind::VoltageRating,
        FactKind::MatingCycles,
        FactKind::MountingStyle,
        FactKind::Orientation,
        FactKind::LockingMechanism,
        FactKind::Shielding,
        FactKind::ImpedanceControlled,
        FactKind::DifferentialPairs,
        FactKind::DeviceType,
        FactKind::ActuationForce,
        FactKind::ActuationTravel,
        FactKind::MechanicalLife,
        FactKind::ElectricalLife,
        FactKind::ContactRatingVoltage,
        FactKind::ContactRatingCurrent,
        FactKind::CoilVoltage,
        FactKind::CoilPower,
        FactKind::ContactForm,
        FactKind::ResolutionEncoder,
        FactKind::Airflow,
        FactKind::StaticPressure,
        FactKind::Torque,
        FactKind::Speed,
    ];

    for kind in all_kinds {
        // Skip if neither metadata nor datasheet has this kind
        let meta_value = get_metadata_accessor(kind, component_class).and_then(|f| f(metadata));
        let ds_facts = facts.get_facts_by_kind(kind);
        let ds_value = ds_facts.first().map(|f| f.parameter.clone());

        if meta_value.is_none() && ds_value.is_none() {
            continue; // Skip fields neither has
        }

        checked_kinds.insert(kind);

        let comparison = match (&meta_value, &ds_value) {
            (Some(m), Some(d)) => match compare_facts(m, d) {
                Ok(ComparisonType::Match) => FieldComparisonResult {
                    fact_kind: kind,
                    name: DatasheetFact::kind_description(kind).to_string(),
                    metadata_value: Some(m.clone()),
                    datasheet_value: Some(d.clone()),
                    comparison: ComparisonType::Match,
                    confidence: 1.0,
                    details: format!("Both: {}", m.value),
                },
                Ok(ComparisonType::Conflict) => FieldComparisonResult {
                    fact_kind: kind,
                    name: DatasheetFact::kind_description(kind).to_string(),
                    metadata_value: Some(m.clone()),
                    datasheet_value: Some(d.clone()),
                    comparison: ComparisonType::Conflict,
                    confidence: 1.0,
                    details: format!("Metadata: {} vs Datasheet: {}", m.value, d.value),
                },
                Ok(other) => FieldComparisonResult {
                    fact_kind: kind,
                    name: DatasheetFact::kind_description(kind).to_string(),
                    metadata_value: Some(m.clone()),
                    datasheet_value: Some(d.clone()),
                    comparison: other,
                    confidence: 0.5,
                    details: format!("Comparison: {:?}", other),
                },
                Err(_) => FieldComparisonResult {
                    fact_kind: kind,
                    name: DatasheetFact::kind_description(kind).to_string(),
                    metadata_value: Some(m.clone()),
                    datasheet_value: Some(d.clone()),
                    comparison: ComparisonType::Incomparable,
                    confidence: 0.0,
                    details: "Dimension mismatch or comparison error".to_string(),
                },
            },
            (Some(m), None) => FieldComparisonResult {
                fact_kind: kind,
                name: DatasheetFact::kind_description(kind).to_string(),
                metadata_value: Some(m.clone()),
                datasheet_value: None,
                comparison: ComparisonType::MissingInDatasheet,
                confidence: 0.8,
                details: format!("Metadata has: {}, datasheet missing", m.value),
            },
            (None, Some(d)) => FieldComparisonResult {
                fact_kind: kind,
                name: DatasheetFact::kind_description(kind).to_string(),
                metadata_value: None,
                datasheet_value: Some(d.clone()),
                comparison: ComparisonType::DatasheetOnly,
                confidence: 0.8,
                details: format!("Datasheet has: {}, metadata missing", d.value),
            },
            (None, None) => continue, // Already filtered
        };

        field_comparisons.push(comparison);
    }

    // Build summary
    let summary = CrossCheckSummary {
        total_fields_checked: field_comparisons.len(),
        matches: field_comparisons
            .iter()
            .filter(|f| f.comparison == ComparisonType::Match)
            .count(),
        conflicts: field_comparisons
            .iter()
            .filter(|f| f.comparison == ComparisonType::Conflict)
            .count(),
        missing_in_datasheet: field_comparisons
            .iter()
            .filter(|f| f.comparison == ComparisonType::MissingInDatasheet)
            .count(),
        datasheet_only: field_comparisons
            .iter()
            .filter(|f| f.comparison == ComparisonType::DatasheetOnly)
            .count(),
        both_missing: field_comparisons
            .iter()
            .filter(|f| f.comparison == ComparisonType::BothMissing)
            .count(),
        incomparable: field_comparisons
            .iter()
            .filter(|f| f.comparison == ComparisonType::Incomparable)
            .count(),
    };

    // Determine assessment
    let assessment = if summary.conflicts > 0 {
        if summary.conflicts > 3 {
            CrossCheckAssessment::Suspicious
        } else {
            CrossCheckAssessment::HasConflicts
        }
    } else if summary.missing_in_datasheet > 0 {
        CrossCheckAssessment::Incomplete
    } else {
        CrossCheckAssessment::Clean
    };

    CrossCheckReport {
        component_mpn: component_mpn.to_string(),
        component_manufacturer: component_manufacturer.to_string(),
        component_class,
        field_comparisons,
        summary,
        assessment,
        checked_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// High-level function to run cross-check and return report.
pub fn run_cross_check(
    component_mpn: &str,
    component_manufacturer: &str,
    component_class: ComponentClass,
    metadata: &ComponentMetadata,
    facts: &FactCollection,
) -> CrossCheckReport {
    cross_check_metadata(
        component_mpn,
        component_manufacturer,
        component_class,
        metadata,
        facts,
    )
}

impl ComponentMetadata {
    pub fn with_family(class: ComponentClass) -> Self {
        Self {
            core: ComponentCoreMetadata::default(),
            family: Some(FamilyMetadata::default_for(class)),
        }
    }

    pub fn get_family_metadata(&self, class: ComponentClass) -> Option<&FamilyMetadata> {
        self.family
            .as_ref()
            .filter(|f| f.component_class() == class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{DatasheetFact, FactCollection, FactKind, ParameterFact, ProvenanceInfo};
    use eak_domain::ComponentClass;
    use eak_units::{PhysicalQuantity, Unit};

    fn create_test_metadata() -> ComponentMetadata {
        let mut metadata = ComponentMetadata::with_family(ComponentClass::Resistor);
        metadata.core.max_operating_voltage = Some(PhysicalQuantity::new(50.0, Unit::Volt));
        metadata.core.max_power_dissipation = Some(PhysicalQuantity::new(0.125, Unit::Watt));
        // Note: Not setting core temperature to test datasheet-only temperature range

        if let Some(FamilyMetadata::Passive(ref mut p)) = metadata.family {
            p.resistance = Some(PhysicalQuantity::new(10_000.0, Unit::Ohm));
            p.power_rating = Some(PhysicalQuantity::new(0.125, Unit::Watt));
            p.temperature_coefficient = Some(PhysicalQuantity::new(100.0, Unit::Unitless));
            // ppm/°C
        }

        metadata
    }

    fn create_test_facts() -> FactCollection {
        let mut coll = FactCollection::new(1, "1.0.0".to_string());

        // Matching resistance
        coll.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Resistance,
            parameter: ParameterFact::new(PhysicalQuantity::new(10_000.0, Unit::Ohm)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });

        // Matching voltage
        coll.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Voltage,
            parameter: ParameterFact::new(PhysicalQuantity::new(50.0, Unit::Volt)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });

        // Conflicting power rating
        coll.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Power,
            parameter: ParameterFact::new(PhysicalQuantity::new(0.25, Unit::Watt)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });

        // Datasheet-only temperature range
        coll.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::TemperatureMin,
            parameter: ParameterFact::new(PhysicalQuantity::new(-40.0, Unit::DegreeCelsius)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });

        coll.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::TemperatureMax,
            parameter: ParameterFact::new(PhysicalQuantity::new(125.0, Unit::DegreeCelsius)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });

        coll
    }

    #[test]
    fn test_cross_check_match() {
        let metadata = create_test_metadata();
        let facts = create_test_facts();

        let report = cross_check_metadata(
            "RC0402",
            "Yageo",
            ComponentClass::Resistor,
            &metadata,
            &facts,
        );

        assert!(report.summary.matches > 0);
        assert!(report.summary.conflicts > 0); // Power rating conflict
        assert!(report.summary.datasheet_only > 0); // Temperature range
        assert_eq!(report.assessment, CrossCheckAssessment::HasConflicts);
    }

    #[test]
    fn test_cross_check_clean() {
        // Create metadata without temperature to avoid missing_in_datasheet
        let mut metadata = ComponentMetadata::with_family(ComponentClass::Resistor);
        metadata.core.max_operating_voltage = Some(PhysicalQuantity::new(50.0, Unit::Volt));
        metadata.core.max_power_dissipation = Some(PhysicalQuantity::new(0.25, Unit::Watt));

        if let Some(FamilyMetadata::Passive(ref mut p)) = metadata.family {
            p.resistance = Some(PhysicalQuantity::new(10_000.0, Unit::Ohm));
            p.power_rating = Some(PhysicalQuantity::new(0.25, Unit::Watt));
        }

        let mut facts = FactCollection::new(1, "1.0.0".to_string());
        facts.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Resistance,
            parameter: ParameterFact::new(PhysicalQuantity::new(10_000.0, Unit::Ohm)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });
        facts.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Voltage,
            parameter: ParameterFact::new(PhysicalQuantity::new(50.0, Unit::Volt)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });
        facts.add_fact(DatasheetFact {
            id: None,
            asset_id: 1,
            kind: FactKind::Power,
            parameter: ParameterFact::new(PhysicalQuantity::new(0.25, Unit::Watt)),
            provenance: ProvenanceInfo::default(),
            created_at: String::new(),
            updated_at: String::new(),
        });

        let report = cross_check_metadata(
            "RC0402",
            "Yageo",
            ComponentClass::Resistor,
            &metadata,
            &facts,
        );

        assert_eq!(report.assessment, CrossCheckAssessment::Clean);
        assert_eq!(report.summary.conflicts, 0);
    }

    #[test]
    fn test_compare_facts_same_value() {
        let m = ParameterFact::new(PhysicalQuantity::new(3.3, Unit::Volt));
        let d = ParameterFact::new(PhysicalQuantity::new(3300.0, Unit::Millivolt));
        assert_eq!(compare_facts(&m, &d).unwrap(), ComparisonType::Match);
    }

    #[test]
    fn test_compare_facts_conflict() {
        let m = ParameterFact::new(PhysicalQuantity::new(3.3, Unit::Volt));
        let d = ParameterFact::new(PhysicalQuantity::new(5.0, Unit::Volt));
        assert_eq!(compare_facts(&m, &d).unwrap(), ComparisonType::Conflict);
    }

    #[test]
    fn test_compare_facts_dimension_mismatch() {
        let m = ParameterFact::new(PhysicalQuantity::new(3.3, Unit::Volt));
        let d = ParameterFact::new(PhysicalQuantity::new(10.0, Unit::Ampere));
        assert_eq!(compare_facts(&m, &d).unwrap(), ComparisonType::Incomparable);
    }
}

// Default implementation for ProvenanceInfo
impl Default for ProvenanceInfo {
    fn default() -> Self {
        Self {
            asset_id: 0,
            asset_sha256: String::new(),
            page: 0,
            extracted_text: String::new(),
            parser_version: String::new(),
            extracted_at: chrono::Utc::now().to_rfc3339(),
            verified: false,
            manually_corrected: false,
            correction_notes: None,
        }
    }
}
