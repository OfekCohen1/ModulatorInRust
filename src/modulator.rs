/// The common interface for all modulation techniques.
pub trait Modulator {
    /// Modulates an entire buffer of message samples.
    ///
    /// * `message` - The input signal (message) to modulate onto the carrier.
    /// * `sample_rate` - The system sample rate in Hz.
    fn modulate(&mut self, message: &[f64], sample_rate: f64) -> Vec<f64>;
}

/// A standard Amplitude Modulator (AM).
///
/// Standard AM uses a "DC offset" (the 1.0 in the formula) to ensure the
/// carrier is always present, which simplifies envelope demodulation.
pub struct AmModulator {
    /// The frequency of the carrier sine wave in Hz.
    pub carrier_freq: f64,
    /// The modulation index (0.0 to 1.0). Determines the "depth" of the modulation.
    pub modulation_index: f64,
}
// Hello World My Name is Ofek
impl AmModulator {
    /// Creates a new AM Modulator.
    pub fn new(carrier_freq: f64, modulation_index: f64) -> Self {
        Self {
            carrier_freq,
            modulation_index,
        }
    }
}

impl Modulator for AmModulator {
    fn modulate(&mut self, message: &[f64], sample_rate: f64) -> Vec<f64> {
        let two_pi = 2.0 * std::f64::consts::PI;

        message
            .iter()
            .enumerate()
            .map(|(i, &msg_sample)| {
                // Calculate time t for the current sample index
                let t = i as f64 / sample_rate;

                // Generate the carrier signal at time t
                // We use cos() as the standard reference for carriers
                let carrier = (two_pi * self.carrier_freq * t).cos();

                // Apply the AM formula: s(t) = [1 + m * m(t)] * c(t)
                let envelope = 1.0 + (self.modulation_index * msg_sample);

                envelope * carrier
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    /// Fixture: Provides a standard sample rate for all tests.
    #[fixture]
    fn sample_rate() -> f64 {
        1000.0
    }

    #[rstest]
    #[case(100.0, 1.0)]  // 100% modulation
    #[case(100.0, 0.5)]  // 50% modulation
    #[case(440.0, 0.2)]  // Subtle modulation at higher freq
    fn test_am_modulation_full_signal(
        #[case] freq: f64,
        #[case] index: f64,
        sample_rate: f64
    ) {
        // --- Arrange ---
        let mut am = AmModulator::new(freq, index);
        let message = vec![1.0; 50]; 
        let pi2 = 2.0 * std::f64::consts::PI;
        let expected_signal = (0..message.len()).map(|i| {
            let t = i as f64 / sample_rate;
            (1.0 + index) * (pi2 * freq * t).cos()
        });

        // --- Act ---
        let output = am.modulate(&message, sample_rate);

        // --- Assert ---
        output.iter().zip(expected_signal).enumerate().for_each(|(i, (actual, expected))| {
            assert!((actual - expected).abs() < 1e-10, 
                "Sample {} mismatch. Actual: {}, Expected: {}", i, actual, expected);
        });
    }

    #[rstest]
    fn test_am_neutral_message(sample_rate: f64) {
        // --- Arrange ---
        let mut am = AmModulator::new(200.0, 0.8);
        let message = vec![0.0; 100]; 
        let pi2 = 2.0 * std::f64::consts::PI;
        let expected_carrier = (0..message.len()).map(|i| {
            let t = i as f64 / sample_rate;
            (pi2 * 200.0 * t).cos()
        });

        // --- Act ---
        let output = am.modulate(&message, sample_rate);

        // --- Assert ---
        assert!(output.iter().zip(expected_carrier).all(|(actual, expected)| {
            (actual - expected).abs() < 1e-10
        }), "Output should be identical to the carrier when message is silent");
    }

    #[rstest]
    fn test_am_pinch_point(sample_rate: f64) {
        // --- Arrange ---
        let mut am = AmModulator::new(100.0, 1.0);
        let message = vec![-1.0; 100];

        // --- Act ---
        let output = am.modulate(&message, sample_rate);

        // --- Assert ---
        assert!(output.iter().all(|&val| val.abs() < 1e-10), 
            "Signal should be completely zeroed out when envelope hits 0.0");
    }
}
