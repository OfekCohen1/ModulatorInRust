use modulator_in_rust::modulator::{AmModulator, Modulator};
use modulator_in_rust::demodulator::{AmCoherentDetector, Demodulator};
use modulator_in_rust::signal_dumper::dump_signal;
use std::f64::consts::PI;

/// System-wide constants for the demonstration.
const SAMPLE_RATE: f64 = 1000.0;
const DURATION: f64 = 0.5;
const MESSAGE_FREQUENCY: f64 = 5.0;
const CARRIER_FREQUENCY: f64 = 100.0;
const MODULATION_INDEX: f64 = 0.8;

/// Demodulation constants
const FILTER_CUTOFF_FREQ: f64 = 15.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting AM Modulation/Demodulation Demonstration...");
    let num_samples = (SAMPLE_RATE * DURATION) as usize;

    // 1. Generate the Message Signal (5Hz Sine Wave)
    println!("Generating {}Hz message signal...", MESSAGE_FREQUENCY);
    let message_signal: Vec<f64> = (0..num_samples)
        .map(|i| {
            let t = i as f64 / SAMPLE_RATE;
            (2.0 * PI * MESSAGE_FREQUENCY * t).sin()
        })
        .collect();

    // 2. Perform Amplitude Modulation (AM)
    println!(
        "Modulating with {}Hz carrier (Index: {})...",
        CARRIER_FREQUENCY, MODULATION_INDEX
    );
    let mut am_modulator = AmModulator::new(CARRIER_FREQUENCY, MODULATION_INDEX, SAMPLE_RATE);
    let mut am_signal = vec![0.0; num_samples];
    am_modulator.modulate(&message_signal, &mut am_signal);

    // 3. Perform AM Coherent Demodulation
    println!("Demodulating signal...");
    let mut detector = AmCoherentDetector::new(CARRIER_FREQUENCY, MODULATION_INDEX, FILTER_CUTOFF_FREQ, SAMPLE_RATE);
    let mut recovered_signal = vec![0.0; num_samples];
    detector.demodulate(&am_signal, &mut recovered_signal);

    // 4. Signal Dumping (External Diagnostic Analysis)
    println!("Dumping signals for external Python analysis...");

    // Stage 1: Message Analysis
    dump_signal("original_message", &message_signal);

    // Stage 2: AM Signal Analysis
    dump_signal("am_modulated_signal", &am_signal);

    // Stage 3: Recovered Signal Analysis
    dump_signal("recovered_message", &recovered_signal);

    println!("Demo complete. Run Python plotters in /tools/ to visualize signals.");

    Ok(())
}
