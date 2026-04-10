use crate::convolve::{Convolver, DirectConvolver, ConvolveMode};
use std::borrow::Cow;
use std::f64::consts::PI;

/// The common interface for all modulation techniques.
pub trait Modulator {
    /// Modulates an entire buffer of message samples.
    ///
    /// # Arguments
    /// * `message` - The input signal (message) to modulate onto the carrier.
    /// * `output` - The pre-allocated buffer to store the modulated signal.
    fn modulate(&mut self, message: &[f64], output: &mut [f64]);
}

/// Supported pulse shapes for digital modulation.
pub enum PulseShape {
    /// A simple rectangular pulse (no shaping).
    Rectangular,
}

/// A standard Amplitude Modulator (AM) with automatic normalization safety.
pub struct AmModulator {
    /// The frequency of the carrier sine wave in Hz.
    pub carrier_frequency: f64,
    /// The modulation index (0.0 to 1.0). Determines the "depth" of the modulation.
    pub modulation_index: f64,
    /// The system sample rate in Hz.
    pub sample_rate: f64,
}

impl AmModulator {
    /// Creates a new AM Modulator.
    pub fn new(carrier_frequency: f64, modulation_index: f64, sample_rate: f64) -> Self {
        Self {
            carrier_frequency,
            modulation_index,
            sample_rate,
        }
    }

    /// Internal helper to ensure the message is within the safe range [-1.0, 1.0].
    ///
    /// If the signal exceeds 1.0, it is normalized to prevent overmodulation.
    /// We use Cow (Clone-on-Write) to avoid allocating memory if the signal is already safe.
    fn get_safe_message<'a>(&self, message: &'a [f64]) -> Cow<'a, [f64]> {
        // Find the peak absolute value using a functional fold.
        let peak = message
            .iter()
            .fold(0.0f64, |max_abs, &val| max_abs.max(val.abs()));

        if peak > 1.0 {
            println!("Warning: Message signal peak ({:.2}) exceeds 1.0. Normalizing to avoid overmodulation.", peak);
            let normalized: Vec<f64> = message.iter().map(|&val| val / peak).collect();
            Cow::Owned(normalized)
        } else {
            Cow::Borrowed(message)
        }
    }
}

impl Modulator for AmModulator {
    fn modulate(&mut self, message: &[f64], output: &mut [f64]) {
        assert_eq!(message.len(), output.len(), "Message and output buffers must have the same length");
        
        let safe_message = self.get_safe_message(message);
        let angular_frequency_per_sample = 2.0 * PI * self.carrier_frequency / self.sample_rate;

        output.iter_mut().enumerate().zip(safe_message.iter()).for_each(|((i, out), &msg_sample)| {
            // Generate carrier: cos(2 * PI * f * t)
            let carrier = (angular_frequency_per_sample * i as f64).cos();

            // AM formula: s(t) = [1 + m * m(t)] * c(t)
            let envelope = 1.0 + (self.modulation_index * msg_sample);

            *out = envelope * carrier;
        });
    }
}

/// A Binary Phase Shift Keying (BPSK) Modulator with pulse shaping.
pub struct BpskModulator {
    pub carrier_frequency: f64,
    pub symbol_rate: f64,
    pub sample_rate: f64,
    pub samples_per_symbol: usize,
    pub baseband_pulse: Vec<f64>,
    /// Pre-allocated buffer for zero-stuffing (upsampling).
    workspace_zero_stuffed: Vec<f64>,
    /// Pre-allocated buffer for the filtered baseband signal.
    workspace_baseband: Vec<f64>,
}

impl BpskModulator {
    /// Creates a new BPSK Modulator.
    ///
    /// Enforces that the sample rate is an integer multiple of the symbol rate.
    pub fn new(
        carrier_frequency: f64,
        symbol_rate: f64,
        sample_rate: f64,
        shape: PulseShape,
        max_message_len: usize,
    ) -> Self {
        // Requirement: Ensure sps is an integer
        assert_eq!((sample_rate / symbol_rate).fract(), 0.0, "Sample rate ({}) must be an integer multiple of symbol rate ({})", sample_rate, symbol_rate);
        let samples_per_symbol = (sample_rate / symbol_rate) as usize;

        let baseband_pulse = match shape {
            PulseShape::Rectangular => vec![1.0; samples_per_symbol],
        };

        let max_samples = max_message_len * samples_per_symbol;

        Self {
            carrier_frequency,
            symbol_rate,
            sample_rate,
            samples_per_symbol,
            baseband_pulse,
            workspace_zero_stuffed: vec![0.0; max_samples],
            workspace_baseband: vec![0.0; max_samples],
        }
    }
}

impl Modulator for BpskModulator {
    fn modulate(&mut self, message: &[f64], output: &mut [f64]) {
        let expected_len = message.len() * self.samples_per_symbol;
        assert_eq!(output.len(), expected_len, "Output buffer must be exactly {} samples for {} message bits", expected_len, message.len());
        assert!(expected_len <= self.workspace_zero_stuffed.len(), "Message exceeds pre-allocated workspace capacity");

        let angular_freq = 2.0 * PI * self.carrier_frequency / self.sample_rate;

        // 1. Zero-stuffing (upsampling) using pre-allocated workspace
        self.workspace_zero_stuffed[..expected_len].fill(0.0);
        
        let pulse_length = self.baseband_pulse.len();
        let half_pulse_length = (pulse_length - 1) / 2; // Same mode offset

        message.iter().enumerate().for_each(|(i, &bit)| {
            let bit_val = if bit == 1.0 { 1.0 } else { -1.0 };
            let pos = i * self.samples_per_symbol + half_pulse_length;
            if pos < expected_len {
                self.workspace_zero_stuffed[pos] = bit_val;
            }
        });

        // 2. Pulse Shaping via Convolution using pre-allocated workspace
        let convolver = DirectConvolver;
        let output_baseband_slice = &mut self.workspace_baseband[..expected_len];
        convolver.convolve(
            &self.workspace_zero_stuffed[..expected_len], 
            &self.baseband_pulse, 
            output_baseband_slice, 
            ConvolveMode::Same
        );

        // 3. Carrier Multiplication (Passband translation)
        output.iter_mut()
            .zip(output_baseband_slice.iter())
            .enumerate()
            .for_each(|(i, (out, &bb_sample))| {
                let carrier = (angular_freq * i as f64).cos();
                *out = bb_sample * carrier;
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    const EPSILON: f64 = 1e-10;

    #[fixture]
    fn sample_rate() -> f64 {
        1000.0
    }

    #[rstest]
    fn test_am_normalization_safety(sample_rate: f64) {
        // --- Arrange ---
        let mut am = AmModulator::new(100.0, 1.0, sample_rate);
        let oversized_message = vec![2.0; 100];
        let mut output = vec![0.0; 100];

        // --- Act ---
        am.modulate(&oversized_message, &mut output);

        // --- Assert ---
        let angular_frequency_per_sample = 2.0 * PI * 100.0 / sample_rate;
        output.iter().enumerate().for_each(|(i, &val)| {
            let expected = 2.0 * (angular_frequency_per_sample * i as f64).cos();
            assert!((val - expected).abs() < EPSILON);
        });
    }

    #[rstest]
    #[case(100.0, 1.0)]
    #[case(440.0, 0.5)]
    fn test_am_modulation_full_signal(
        #[case] carrier_frequency: f64,
        #[case] modulation_index: f64,
        sample_rate: f64,
    ) {
        // --- Arrange ---
        let mut am = AmModulator::new(carrier_frequency, modulation_index, sample_rate);
        let message = vec![1.0; 50];
        let mut output = vec![0.0; 50];

        let angular_frequency_per_sample = 2.0 * PI * carrier_frequency / sample_rate;
        let expected_signal = (0..message.len())
            .map(|i| (1.0 + modulation_index) * (angular_frequency_per_sample * i as f64).cos());

        // --- Act ---
        am.modulate(&message, &mut output);

        // --- Assert ---
        output
            .iter()
            .zip(expected_signal)
            .enumerate()
            .for_each(|(i, (actual, expected))| {
                assert!((actual - expected).abs() < EPSILON, "Sample {} mismatch", i);
            });
    }

    #[rstest]
    fn test_bpsk_integer_sps_enforcement() {
        // --- Arrange ---
        // 1000 / 333 is not an integer

        // --- Act ---
        let result = std::panic::catch_unwind(|| {
            BpskModulator::new(100.0, 333.0, 1000.0, PulseShape::Rectangular, 10);
        });

        // --- Assert ---
        assert!(result.is_err());
    }

    #[rstest]
    fn test_bpsk_modulation_output(sample_rate: f64) {
        // --- Arrange ---
        let carrier_freq = 100.0;
        let symbol_rate = 100.0; // 10 samples per symbol
        let mut bpsk = BpskModulator::new(
            carrier_freq,
            symbol_rate,
            sample_rate,
            PulseShape::Rectangular,
            2,
        );

        let message = vec![1.0, 0.0];
        let mut output = vec![0.0; 20];
        let angular_freq = 2.0 * PI * carrier_freq / sample_rate;

        // Generate expected result beforehand using iterators
        let expected: Vec<f64> = (0..20)
            .map(|i| {
                let bit_val = if i < 10 { 1.0 } else { -1.0 };
                bit_val * (angular_freq * i as f64).cos()
            })
            .collect();

        // --- Act ---
        bpsk.modulate(&message, &mut output);

        // --- Assert ---
        assert_eq!(output.len(), 20);
        output
            .iter()
            .zip(expected.iter())
            .enumerate()
            .for_each(|(i, (actual, exp))| {
                assert!((actual - exp).abs() < EPSILON, "Sample {} mismatch", i);
            });
    }
}
