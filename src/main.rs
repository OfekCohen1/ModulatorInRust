mod demodulator;
mod modulator;
mod plotter;

use modulator::{AmModulator, Modulator};
use plotter::PlotWrapper;
use std::f64::consts::PI;

/// System-wide constants for the demonstration.
const SAMPLE_RATE: f64 = 1000.0;
const DURATION: f64 = 0.5;
const MESSAGE_FREQUENCY: f64 = 5.0;
const CARRIER_FREQUENCY: f64 = 100.0;
const MODULATION_INDEX: f64 = 0.8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting AM Modulation Demonstration...");
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

    // 3. Visualization using Subplots (2x1 Grid)
    println!("Preparing plots...");
    let mut plot_wrapper = PlotWrapper::new(2, 1);
    plot_wrapper.set_title("Amplitude Modulation (AM) Demonstration");

    // Plot 1: The Message Signal (Row 1, Col 1)
    plot_wrapper.add_signal(1, 1, "Message (5Hz)", &message_signal, SAMPLE_RATE);

    // Plot 2: The Modulated Signal (Row 2, Col 1)
    plot_wrapper.add_signal(2, 1, "AM Signal (100Hz Carrier)", &am_signal, SAMPLE_RATE);

    // TODO: Next time - Perform AM Coherent Demodulation and add a 3rd plot to the gallery.

    println!("Opening AM demonstration in browser...");
    plot_wrapper.show();

    Ok(())
}
