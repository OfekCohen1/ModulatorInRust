use std::f64::consts::PI;
use std::borrow::Cow;

/// The common interface for all modulation techniques.
pub trait Modulator {
    /// Modulates an entire buffer of message samples.
    ///
    /// # Arguments
    /// * `message` - The input signal (message) to modulate onto the carrier.
    /// * `sample_rate` - The system sample rate in Hz.
    fn modulate(&mut self, message: &[f64], sample_rate: f64) -> Vec<f64>;
}

/// A standard Amplitude Modulator (AM) with automatic normalization safety.
pub struct AmModulator {
    /// The frequency of the carrier sine wave in Hz.
    pub carrier_frequency: f64,
    /// The modulation index (0.0 to 1.0). Determines the "depth" of the modulation.
    pub modulation_index: f64,
}

impl AmModulator {
    /// Creates a new AM Modulator.
    pub fn new(carrier_frequency: f64, modulation_index: f64) -> Self {
        Self {
            carrier_frequency,
            modulation_index,
        }
    }

    /// Internal helper to ensure the message is within the safe range [-1.0, 1.0].
    /// 
    /// If the signal exceeds 1.0, it is normalized to prevent overmodulation.
    /// We use Cow (Clone-on-Write) to avoid allocating memory if the signal is already safe.
    fn get_safe_message<'a>(&self, message: &'a [f64]) -> Cow<'a, [f64]> {
        // Find the peak absolute value using a functional fold.
        let peak = message.iter().fold(0.0f64, |max_abs, &val| max_abs.max(val.abs()));

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
    fn modulate(&mut self, message: &[f64], sample_rate: f64) -> Vec<f64> {
        let safe_message = self.get_safe_message(message);
        let angular_frequency_per_sample = 2.0 * PI * self.carrier_frequency / sample_rate;

        safe_message
            .iter()
            .enumerate()
            .map(|(i, &msg_sample)| {
                // Generate carrier: cos(2 * PI * f * t)
                let carrier = (angular_frequency_per_sample * i as f64).cos();
                
                // AM formula: s(t) = [1 + m * m(t)] * c(t)
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

    const EPSILON: f64 = 1e-10;

    #[fixture]
    fn sample_rate() -> f64 {
        1000.0
    }

    #[rstest]
    fn test_am_normalization_safety(sample_rate: f64) {
        // --- Arrange ---
        let mut am = AmModulator::new(100.0, 1.0);
        let oversized_message = vec![2.0; 100]; // Peak is 2.0, should be normalized to 1.0

        // --- Act ---
        let output = am.modulate(&oversized_message, sample_rate);

        // --- Assert ---
        // If normalized correctly, envelope is 1 + 1.0 * (2.0/2.0) = 2.0.
        // If NOT normalized, envelope would be 1 + 1.0 * 2.0 = 3.0.
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
        sample_rate: f64
    ) {
        let mut am = AmModulator::new(carrier_frequency, modulation_index);
        let message = vec![1.0; 50]; 
        
        let angular_frequency_per_sample = 2.0 * PI * carrier_frequency / sample_rate;
        let expected_signal = (0..message.len()).map(|i| {
            (1.0 + modulation_index) * (angular_frequency_per_sample * i as f64).cos()
        });

        let output = am.modulate(&message, sample_rate);

        output.iter()
            .zip(expected_signal)
            .enumerate()
            .for_each(|(i, (actual, expected))| {
                assert!((actual - expected).abs() < EPSILON, "Sample {} mismatch", i);
            });
    }
}
