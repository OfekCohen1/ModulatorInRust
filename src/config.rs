use crate::pulse::PulseShape;

/// Fundamental physical parameters shared by all modulation types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseConfig {
    /// Carrier frequency in Hz.
    pub carrier_frequency: f64,
    /// System sample rate in Hz.
    pub sample_rate: f64,
}

impl BaseConfig {
    pub fn new(carrier_frequency: f64, sample_rate: f64) -> Self {
        Self {
            carrier_frequency,
            sample_rate,
        }
    }
}

/// Specialized configuration for Amplitude Modulation (AM).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmConfig {
    pub base: BaseConfig,
    /// Modulation index (0.0 to 1.0).
    pub modulation_index: f64,
}

impl AmConfig {
    pub fn new(carrier_frequency: f64, sample_rate: f64, modulation_index: f64) -> Self {
        Self {
            base: BaseConfig::new(carrier_frequency, sample_rate),
            modulation_index,
        }
    }
}

/// Specialized configuration for Digital Modulation (BPSK, QPSK, etc.).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DigitalConfig {
    pub base: BaseConfig,
    /// Symbol rate in Hz.
    pub symbol_rate: f64,
    /// The pulse shape used for baseband signal conditioning.
    pub pulse_shape: PulseShape,
}

impl DigitalConfig {
    pub fn new(carrier_frequency: f64, symbol_rate: f64, sample_rate: f64, pulse_shape: PulseShape) -> Self {
        // Enforce SPS as integer at the config level
        assert_eq!(
            (sample_rate / symbol_rate).fract(), 
            0.0, 
            "Sample rate ({}) must be an integer multiple of symbol rate ({})", 
            sample_rate, 
            symbol_rate
        );

        Self {
            base: BaseConfig::new(carrier_frequency, sample_rate),
            symbol_rate,
            pulse_shape,
        }
    }

    pub fn samples_per_symbol(&self) -> usize {
        (self.base.sample_rate / self.symbol_rate) as usize
    }
}
