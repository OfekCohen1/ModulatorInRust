// --- REAL IMPLEMENTATION (Only compiled when feature is active) ---
#[cfg(feature = "diagnostic-plots")]
mod imp {
    use plotly::{Plot, Scatter, Layout};
    use plotly::common::Title;
    use plotly::layout::{LayoutGrid, GridPattern};
    use realfft::RealFftPlanner;

    /// Internal helper to create a 2x1 grid plot layout.
    fn create_diagnostic_plot(title: &str) -> Plot {
        let mut plot = Plot::new();
        let layout = Layout::new()
            .title(Title::with_text(title))
            .grid(
                LayoutGrid::new()
                    .rows(2)
                    .columns(1)
                    .pattern(GridPattern::Independent),
            );
        plot.set_layout(layout);
        plot
    }

    /// Adds a time-domain trace to the provided plot (Target: Row 1).
    fn add_time_trace(plot: &mut Plot, name: &str, data: &[f64], sample_rate: f64) {
        let time: Vec<f64> = (0..data.len())
            .map(|i| i as f64 / sample_rate)
            .collect();

        let trace = Scatter::new(time, data.to_vec())
            .name(format!("{} (Time)", name))
            .x_axis("x")
            .y_axis("y");
        
        plot.add_trace(trace);
    }

    /// Adds a frequency-domain trace to the provided plot (Target: Row 2).
    fn add_fft_trace(plot: &mut Plot, name: &str, data: &[f64], sample_rate: f64, n_fft: usize) {
        let n_original = data.len();
        let n = n_fft.max(n_original);
        
        let mut planner = RealFftPlanner::<f64>::new();
        let fft_planner = planner.plan_fft_forward(n);

        let mut indata = fft_planner.make_input_vec();
        for val in indata.iter_mut() { *val = 0.0; }
        indata[..n_original].copy_from_slice(data);
        
        let mut spectrum = fft_planner.make_output_vec();
        fft_planner.process(&mut indata, &mut spectrum).expect("FFT failed");

        let magnitudes: Vec<f64> = spectrum
            .iter()
            .map(|c| c.norm() / n_original as f64)
            .collect();

        let bin_resolution = sample_rate / n as f64;
        let frequencies: Vec<f64> = (0..magnitudes.len())
            .map(|i| i as f64 * bin_resolution)
            .collect();

        let trace = Scatter::new(frequencies, magnitudes)
            .name(format!("{} (Freq)", name))
            .x_axis("x2")
            .y_axis("y2");

        plot.add_trace(trace);
    }

    pub fn plot_diagnostic_time_and_fft(name: &str, data: &[f64], sample_rate: f64, n_fft: usize) {
        let mut plot = create_diagnostic_plot(&format!("Diagnostic: {}",  name));
        
        add_time_trace(&mut plot, name, data, sample_rate);
        add_fft_trace(&mut plot, name, data, sample_rate, n_fft);

        plot.show();
    }
}

// --- NO-OP SHIMS (Only compiled when feature is disabled) ---
#[cfg(not(feature = "diagnostic-plots"))]
mod imp {
    #[inline(always)]
    pub fn plot_diagnostic_time_and_fft(_name: &str, _data: &[f64], _sample_rate: f64, _n_fft: usize) {
        // Optimized away by the compiler
    }
}

// Transparently export the selected implementation
pub use imp::*;
