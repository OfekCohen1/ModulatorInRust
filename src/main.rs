use modulator_in_rust::modulator::{AmModulator, Modulator};
use modulator_in_rust::demodulator::{AmCoherentDetector, Demodulator};
use modulator_in_rust::plotter::{plot_diagnostic_time, plot_diagnostic_fft};
use std::f64::consts::PI;

/// System-wide constants for the demonstration.
const SAMPLE_RATE: f64 = 1000.0;
const DURATION: f64 = 0.5;
const MESSAGE_FREQUENCY: f64 = 5.0;
const CARRIER_FREQUENCY: f64 = 100.0;
const MODULATION_INDEX: f64 = 0.8;

/// Demodulation constants
const FILTER_CUTOFF_FREQ: f64 = 15.0;

/// Visualization constants
/// Choosing 2000 gives us a 0.5Hz resolution (1000/2000), 
/// which aligns perfectly with our 5Hz message.
const FFT_SIZE: usize = 2000;

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
    let mut am_modulator = AmModulator::new(CARRIER_FREQUENCY, MODULATION_INDEX);
    let am_signal = am_modulator.modulate(&message_signal, SAMPLE_RATE);

    // 3. Perform AM Coherent Demodulation
    println!("Demodulating signal...");
    let mut detector = AmCoherentDetector::new(CARRIER_FREQUENCY, FILTER_CUTOFF_FREQ, SAMPLE_RATE);
    let recovered_signal = detector.demodulate(&am_signal);

    // 4. Visualization (Standalone Diagnostic Plots)
    println!("Displaying diagnostic plots...");

    // Stage 1: Message Analysis
    plot_diagnostic_time("Original Message", &message_signal, SAMPLE_RATE);
    plot_diagnostic_fft("Original Spectrum", &message_signal, SAMPLE_RATE, FFT_SIZE);

    // Stage 2: AM Signal Analysis
    plot_diagnostic_time("AM Modulated Signal", &am_signal, SAMPLE_RATE);
    plot_diagnostic_fft("AM Spectrum", &am_signal, SAMPLE_RATE, FFT_SIZE);

    // Stage 3: Recovered Signal Analysis
    plot_diagnostic_time("Recovered Message", &recovered_signal, SAMPLE_RATE);
    plot_diagnostic_fft("Recovered Spectrum", &recovered_signal, SAMPLE_RATE, FFT_SIZE);

    println!("Demo complete. Standalone plots opened in browser (if diagnostic-plots feature is enabled).");

    Ok(())
}
