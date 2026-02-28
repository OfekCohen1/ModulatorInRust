use plotly::{Plot, Scatter, Layout};
use plotly::common::{Mode, Title};

/// A simple wrapper around Plotly to make plotting signals easy.
pub struct PlotWrapper {
    plot: Plot,
}

impl PlotWrapper {
    /// Create a new plotter instance.
    pub fn new() -> Self {
        Self {
            plot: Plot::new(),
        }
    }

    /// Set the title of the plot.
    pub fn set_title(&mut self, title: &str) {
        let layout = Layout::new().title(Title::with_text(title));
        self.plot.set_layout(layout);
    }

    /// Add a signal to the plot.
    ///
    /// * `name` - The name of the trace (appears in legend).
    /// * `data` - The signal values.
    /// * `sample_rate` - The sample rate in Hz (used to calculate time on X-axis).
    pub fn add_signal(&mut self, name: &str, data: &[f64], sample_rate: f64) {
        let time: Vec<f64> = (0..data.len())
            .map(|i| i as f64 / sample_rate)
            .collect();

        let trace = Scatter::new(time, data.to_vec())
            .mode(Mode::Lines)
            .name(name);

        self.plot.add_trace(trace);
    }

    /// Add a raw data series (X-axis will be the index).
    pub fn add_raw(&mut self, name: &str, data: &[f64]) {
        let indices: Vec<f64> = (0..data.len())
            .map(|i| i as f64)
            .collect();

        let trace = Scatter::new(indices, data.to_vec())
            .mode(Mode::Lines)
            .name(name);

        self.plot.add_trace(trace);
    }

    /// Show the plot in the default browser.
    pub fn show(&self) {
        self.plot.show();
    }
}
