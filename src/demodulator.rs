//! A module for demodulating communication signals.

use std::f64::consts::PI;
use biquad::{ToHertz, Coefficients, DirectForm2Transposed, Type, Q_BUTTERWORTH_F64, Biquad};
use crate::signal_dumper::dump_signal;

/// The common interface for all demodulation techniques.
pub trait Demodulator {
    /// Demodulates a buffer of signal samples back into message samples.
    ///
    /// # Arguments
    /// * `signal` - The modulated signal to process.
    /// * `output` - The pre-allocated buffer to store the recovered message.
    fn demodulate(&mut self, signal: &[f64], output: &mut [f64]);
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
    /// The modulation index used during modulation (required for amplitude recovery).
    modulation_index: f64,
}

impl AmCoherentDetector {
    /// Creates a new Coherent AM Detector.
    /// 
    /// Pre-calculates both the filter coefficients and the mixing constant.
    pub fn new(carrier_frequency: f64, modulation_index: f64, filter_cutoff_freq: f64, sample_rate: f64) -> Self {
        let angular_frequency_per_sample = 2.0 * PI * carrier_frequency / sample_rate;

        let filter_coefficients = Coefficients::<f64>::from_params(
            Type::LowPass,
            sample_rate.hz(),
            filter_cutoff_freq.hz(),
            Q_BUTTERWORTH_F64,
        ).expect("Invalid filter parameters");

        // --- Diagnostic: Dump the LPF characterization ---
        #[cfg(feature = "dump-signals")]
        {
            let mut impulse = vec![0.0; 1000];
            impulse[0] = 1.0;
            // Create a temporary filter instance to measure impulse response
            let mut measurement_filter = DirectForm2Transposed::<f64>::new(filter_coefficients);
            let response: Vec<f64> = impulse.into_iter().map(|x| measurement_filter.run(x)).collect();
            dump_signal("lpf_impulse_response", &response);
        }

        Self {
            angular_frequency_per_sample,
            filter_coefficients,
            modulation_index,
        }
    }
}

impl Demodulator for AmCoherentDetector {
    fn demodulate(&mut self, signal: &[f64], output: &mut [f64]) {
        assert_eq!(signal.len(), output.len(), "Signal and output buffers must have the same length");

        // 1. Mixing: multiply by synchronized local oscillator directly into output
        output.iter_mut().enumerate().zip(signal.iter()).for_each(|((i, out), &s)| {
            *out = s * (self.angular_frequency_per_sample * i as f64).cos();
        });

        // --- Diagnostic: After Mixer ---
        dump_signal("am_after_mixer", output);

        // 2. Zero-Phase Filtering (filtfilt) - Performed in-place on output
        
        // Pass 1: Forward
        let mut forward_filter = DirectForm2Transposed::<f64>::new(self.filter_coefficients);
        for sample in output.iter_mut() {
            *sample = forward_filter.run(*sample);
        }

        // Pass 2: Backward (reverses the phase shift)
        output.reverse();
        let mut backward_filter = DirectForm2Transposed::<f64>::new(self.filter_coefficients);
        for sample in output.iter_mut() {
            *sample = backward_filter.run(*sample);
        }

        // Restore original order
        output.reverse();

        // 3. Post-processing: Remove DC offset and restore scale
        // Note: For IIR filters, we skip enough samples for the exponential 
        // transients to decay (settling time).
        let transient_skip = 50; 
        let window_len = output.len().saturating_sub(2 * transient_skip);
        
        let mean: f64 = if window_len > 0 {
            output.iter()
                .skip(transient_skip)
                .take(window_len)
                .sum::<f64>() / window_len as f64
        } else {
            output.iter().sum::<f64>() / output.len() as f64
        };

        // Recovery Gain: 2.0 (for mixing) / modulation_index (for original amplitude)
        let recovery_gain = 2.0 / self.modulation_index;

        output.iter_mut().for_each(|sample| {
            *sample = (*sample - mean) * recovery_gain;
        });

        // --- Diagnostic: After LPF ---
        dump_signal("am_after_lpf_final", output);
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
        mod_index: f64,
        filter_cutoff: f64,
        sample_rate: f64,
        num_samples: usize,
        signal_generator: F1,
        expected_generator: F2,
    ) where 
        F1: Fn(f64) -> f64, 
        F2: Fn(f64) -> f64 
    {
        let mut demod = AmCoherentDetector::new(carrier_freq, mod_index, filter_cutoff, sample_rate);
        
        let input_signal: Vec<f64> = (0..num_samples)
            .map(|i| signal_generator(i as f64 / sample_rate))
            .collect();
            
        let expected_output: Vec<f64> = (0..num_samples)
            .map(|i| expected_generator(i as f64 / sample_rate))
            .collect();

        let mut output = vec![0.0; num_samples];
        demod.demodulate(&input_signal, &mut output);

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
            1.0, // DC test, mod_index doesn't change 0 result but required for API
            15.0, 
            sample_rate,
            400,
            |t| (2.0 * PI * carrier_freq * t).cos(), 
            |_| 0.0,
        );
    }

    #[rstest]
    fn test_coherent_sine_recovery(sample_rate: f64) {
        let carrier_freq = 100.0;
        let msg_freq = 5.0;
        let mod_index = 0.8;
        
        run_demodulation_test(
            carrier_freq,
            mod_index,
            20.0,
            sample_rate,
            600,
            move |t| (1.0 + mod_index * (2.0 * PI * msg_freq * t).sin()) * (2.0 * PI * carrier_freq * t).cos(),
            move |t| (2.0 * PI * msg_freq * t).sin(),
        );
    }

    #[rstest]
    fn test_coherent_quadrature_null(sample_rate: f64) {
        let carrier_freq = 100.0;
        run_demodulation_test(
            carrier_freq,
            1.0,
            15.0,
            sample_rate,
            400,
            |t| (2.0 * PI * carrier_freq * t).sin(), 
            |_| 0.0,
        );
    }
}
