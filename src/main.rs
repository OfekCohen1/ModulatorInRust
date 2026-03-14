mod plotter;
mod modulator;

use plotter::PlotWrapper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define signal parameters
    let fs = 1000.0;
    let frequency = 5.0;
    let duration = 2.0;
    let num_samples = (fs * duration) as usize;

    println!("Number of samples: {}", num_samples);

    println!("Generating sine wave at {} Hz with sample rate {} Hz...", frequency, fs);

    // 2. Generate the sine wave data
    let y_axis: Vec<f64> = (0..num_samples)
        .map(|i| {
            let t_axis = i as f64 / fs;
            (2.0 * std::f64::consts::PI * frequency * t_axis).sin()
        })
        .collect();

    // 3. Use the PlotWrapper API
    let mut plot_wrapper = PlotWrapper::new();
    plot_wrapper.set_title("Sine Wave Demo");
    plot_wrapper.add_signal("Sine Wave", &y_axis, fs);

    println!("Opening plot in browser...");
    plot_wrapper.show();

    Ok(())
}
