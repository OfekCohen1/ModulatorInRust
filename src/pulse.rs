//! Pulse shaping and matched filter kernel generation.

/// Supported pulse shapes for digital modulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PulseShape {
    /// A simple rectangular pulse (no shaping).
    Rectangular,
}

impl PulseShape {
    /// Generates the discrete-time impulse response (kernel) for the selected shape.
    ///
    /// # Arguments
    /// * `sample_rate` - System sample rate in Hz.
    /// * `symbol_rate` - Symbol rate in Hz.
    pub fn generate_kernel(&self, sample_rate: f64, symbol_rate: f64) -> Vec<f64> {
        let sps = (sample_rate / symbol_rate) as usize;
        
        match self {
            PulseShape::Rectangular => {
                // A rectangular pulse is just a sequence of ones for the duration of a symbol.
                vec![1.0; sps]
            }
        }
    }
}
