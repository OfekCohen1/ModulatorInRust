use modulator_in_rust::demodulator::{DigitalDemodulator, BpskSlicer, Demodulator};
use modulator_in_rust::modulator::{BpskModulator, Modulator};
use modulator_in_rust::pulse::PulseShape;
use modulator_in_rust::config::DigitalConfig;
use rstest::*;

/// A generic loopback test harness for BPSK.
fn run_bpsk_loopback_test(
    carrier_freq: f64,
    symbol_rate: f64,
    sample_rate: f64,
    bits: Vec<f64>,
) {
    // --- Arrange ---
    let config = DigitalConfig::new(carrier_freq, symbol_rate, sample_rate, PulseShape::Rectangular);
    let num_samples = bits.len() * config.samples_per_symbol();
    
    let mut modulator = BpskModulator::new(config, bits.len());
    
    let mut demodulator = DigitalDemodulator::new(
        config,
        BpskSlicer,
        num_samples,
    );

    let mut modulated_signal = vec![0.0; num_samples];
    let mut recovered_bits = vec![0.0; bits.len()];

    // --- Act ---
    modulator.modulate(&bits, &mut modulated_signal);
    demodulator.demodulate(&modulated_signal, &mut recovered_bits);

    // --- Assert ---
    assert!(
        recovered_bits.iter()
            .zip(bits.iter())
            .all(|(&actual, &expected)| (actual - expected).abs() < f64::EPSILON)
    );
}

#[rstest]
#[case(100.0, 10.0, 1000.0, vec![1.0, 0.0, 1.0, 1.0])]           // Standard
#[case(100.0, 5.0, 1000.0, vec![1.0, 0.0])]                    // High SPS
#[case(400.0, 50.0, 1000.0, vec![0.0, 1.0, 0.0, 1.0])]         // Nyquist Edge
#[case(100.0, 20.0, 1000.0, vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0])] // Alternating
#[case(100.0, 10.0, 1000.0, vec![1.0; 32])]                   // Long Sequence
fn test_bpsk_integration_loopback(
    #[case] fc: f64,
    #[case] rs: f64,
    #[case] fs: f64,
    #[case] bits: Vec<f64>,
) {
    // --- Act & Assert ---
    run_bpsk_loopback_test(fc, rs, fs, bits);
}
