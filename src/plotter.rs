// --- REAL IMPLEMENTATION (Only compiled when feature is active) ---
#[cfg(feature = "diagnostic-plots")]
mod imp {
    use plotly::{Plot, Scatter, Layout};
    use plotly::common::Title;
    use realfft::RealFftPlanner;

    /// Internal helper to create a standard 1x1 plot layout.
    fn create_standalone_plot(title: &str) -> Plot {
        let mut plot = Plot::new();
        let layout = Layout::new().title(Title::with_text(title));
        plot.set_layout(layout);
        plot
    }

    pub fn plot_diagnostic_time(name: &str, data: &[f64], sample_rate: f64) {
        let mut plot = create_standalone_plot(&format!("Diagnostic (Time): {}", name));
        
        let time: Vec<f64> = (0..data.len())
            .map(|i| i as f64 / sample_rate)
            .collect();

        let trace = Scatter::new(time, data.to_vec())
            .name(name);

        plot.add_trace(trace);
        plot.show();
    }

    pub fn plot_diagnostic_fft(name: &str, data: &[f64], sample_rate: f64, n_fft: usize) {
        let mut plot = create_standalone_plot(&format!("Diagnostic (FFT): {}", name));
        
        let n_original = data.len();
        let n = n_fft.max(n_original);
        
        let mut planner = RealFftPlanner::<f64>::new();
        let fft_planner = planner.plan_fft_forward(n);

        let mut indata = fft_planner.make_input_vec();
        // fill with zeros and copy original data
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
            .name(format!("{} (Freq)", name));

        plot.add_trace(trace);
        plot.show();
    }
}

// --- NO-OP SHIMS (Only compiled when feature is disabled) ---
#[cfg(not(feature = "diagnostic-plots"))]
mod imp {
    #[inline(always)]
    pub fn plot_diagnostic_time(_name: &str, _data: &[f64], _sample_rate: f64) {
        // Optimized away by the compiler
    }

    #[inline(always)]
    pub fn plot_diagnostic_fft(_name: &str, _data: &[f64], _sample_rate: f64, _n_fft: usize) {
        // Optimized away by the compiler
    }
}

// Transparently export the selected implementation
pub use imp::*;
