//! A module for demodulating communication signals.

use std::f64::consts::PI;
use biquad::{ToHertz, Coefficients, DirectForm2Transposed, Type, Q_BUTTERWORTH_F64, Biquad};

/// The common interface for all demodulation techniques.
pub trait Demodulator {
    /// Demodulates a buffer of signal samples back into message samples.
    ///
    /// # Arguments
    /// * `signal` - The modulated signal to process.
    fn demodulate(&mut self, signal: &[f64]) -> Vec<f64>;
}

/// A Coherent AM Detector with Zero-Phase filtering.
///
/// This detector uses the "filtfilt" technique (forward-backward filtering)
/// to perfectly cancel the group delay introduced by the IIR filter.
pub struct AmCoherentDetector {
    /// Pre-calculated angular frequency (radians per sample).
    angular_frequency_per_sample: f64,
    /// Pre-calculated filter coefficients.
    filter_coefficients: Coefficients<f64>,
}

impl AmCoherentDetector {
    /// Creates a new Coherent AM Detector.
    /// 
    /// Pre-calculates both the filter coefficients and the mixing constant.
    pub fn new(carrier_frequency: f64, filter_cutoff_freq: f64, sample_rate: f64) -> Self {
        let angular_frequency_per_sample = 2.0 * PI * carrier_frequency / sample_rate;

        let filter_coefficients = Coefficients::<f64>::from_params(
            Type::LowPass,
            sample_rate.hz(),
            filter_cutoff_freq.hz(),
            Q_BUTTERWORTH_F64,
        ).expect("Invalid filter parameters");

        Self {
            angular_frequency_per_sample,
            filter_coefficients,
        }
    }
}

impl Demodulator for AmCoherentDetector {
    fn demodulate(&mut self, signal: &[f64]) -> Vec<f64> {
        // 1. Mixing: multiply by synchronized local oscillator
        let mut data: Vec<f64> = signal
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                sample * (self.angular_frequency_per_sample * i as f64).cos()
            })
            .collect();

        // 2. Zero-Phase Filtering (filtfilt)
        // Pass 1: Forward
        let mut filter = DirectForm2Transposed::<f64>::new(self.filter_coefficients);
        for sample in data.iter_mut() {
            *sample = filter.run(*sample);
        }

        // Pass 2: Backward (reverses the phase shift)
        data.reverse();
        let mut filter = DirectForm2Transposed::<f64>::new(self.filter_coefficients);
        for sample in data.iter_mut() {
            *sample = filter.run(*sample);
        }

        // Restore original order
        data.reverse();
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    use rstest::*;

    // Tolerance for filter ripple and transients
    const EPSILON: f64 = 0.05; 
    const EDGE_OFFSET: usize = 50; // Samples to skip to avoid filter transients

    #[fixture]
    fn sample_rate() -> f64 {
        1000.0
    }

    /// A generic test runner for demodulation scenarios.
    fn run_demodulation_test<F1, F2>(
        carrier_freq: f64,
        filter_cutoff: f64,
        sample_rate: f64,
        num_samples: usize,
        signal_generator: F1,
        expected_generator: F2,
    ) where 
        F1: Fn(f64) -> f64, 
        F2: Fn(f64) -> f64 
    {
        let mut demod = AmCoherentDetector::new(carrier_freq, filter_cutoff, sample_rate);
        
        let input_signal: Vec<f64> = (0..num_samples)
            .map(|i| signal_generator(i as f64 / sample_rate))
            .collect();
            
        let expected_output: Vec<f64> = (0..num_samples)
            .map(|i| expected_generator(i as f64 / sample_rate))
            .collect();

        let output = demod.demodulate(&input_signal);

        output.iter()
            .zip(expected_output.iter())
            .skip(EDGE_OFFSET)
            .take(num_samples - 2 * EDGE_OFFSET)
            .enumerate()
            .for_each(|(i, (actual, expected))| {
                assert!(
                    (actual - expected).abs() < EPSILON,
                    "Mismatch at sample {}. Actual: {}, Expected: {}", 
                    i + EDGE_OFFSET, actual, expected
                );
            });
    }

    #[rstest]
    fn test_coherent_dc_recovery(sample_rate: f64) {
        let carrier_freq = 100.0;
        run_demodulation_test(
            carrier_freq,
            15.0, 
            sample_rate,
            400,
            |t| (2.0 * PI * carrier_freq * t).cos(), 
            |_| 0.5,
        );
    }

    #[rstest]
    fn test_coherent_sine_recovery(sample_rate: f64) {
        let carrier_freq = 100.0;
        let msg_freq = 5.0;
        let mod_index = 0.8;
        
        run_demodulation_test(
            carrier_freq,
            20.0,
            sample_rate,
            600,
            move |t| (1.0 + mod_index * (2.0 * PI * msg_freq * t).sin()) * (2.0 * PI * carrier_freq * t).cos(),
            move |t| 0.5 * (1.0 + mod_index * (2.0 * PI * msg_freq * t).sin()),
        );
    }

    #[rstest]
    fn test_coherent_quadrature_null(sample_rate: f64) {
        let carrier_freq = 100.0;
        run_demodulation_test(
            carrier_freq,
            15.0,
            sample_rate,
            400,
            |t| (2.0 * PI * carrier_freq * t).sin(), 
            |_| 0.0,
        );
    }
}
