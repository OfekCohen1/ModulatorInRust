use plotly::{Plot, Scatter, Layout};
use plotly::common::{Mode, Title};
use plotly::layout::{LayoutGrid, GridPattern};
use realfft::RealFftPlanner;

/// A simple wrapper around Plotly to make plotting signals easy, inspired by MATLAB's subplot logic.
pub struct PlotWrapper {
    plot: Plot,
    rows: usize,
    cols: usize,
}

impl PlotWrapper {
    /// Create a new plotter instance with a fixed grid size.
    ///
    /// * `rows` - Number of vertical plots.
    /// * `cols` - Number of horizontal plots.
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut plot = Plot::new();
        
        // Configure the grid layout immediately.
        // We use Independent pattern so each subplot has its own X and Y scales.
        let layout = Layout::new().grid(
            LayoutGrid::new()
                .rows(rows)
                .columns(cols)
                .pattern(GridPattern::Independent),
        );
        plot.set_layout(layout);

        Self { plot, rows, cols }
    }

    /// Set the main title of the figure.
    pub fn set_title(&mut self, title: &str) {
        let layout = self.plot.layout().clone().title(Title::with_text(title));
        self.plot.set_layout(layout);
    }

    /// Helper to map (row, col) to Plotly axis IDs (e.g., "x", "y" or "x2", "y2").
    fn get_axis_ids_in_plotly_format(&self, row: usize, col: usize) -> (String, String) {
        assert!(row >= 1 && row <= self.rows, "Row {} out of bounds for {} rows", row, self.rows);
        assert!(col >= 1 && col <= self.cols, "Col {} out of bounds for {} cols", col, self.cols);

        // Calculate 1-based index: increments across then down.
        let index = (row - 1) * self.cols + col;

        // Plotly uses "x", "y" for the first axis pair and "x2", "y2" etc. for others.
        if index == 1 {
            ("x".to_string(), "y".to_string())
        } else {
            (format!("x{}", index), format!("y{}", index))
        }
    }

    /// Add a signal to a specific subplot using (row, col) coordinates.
    ///
    /// # Arguments
    /// * `row` - 1-based row index.
    /// * `col` - 1-based column index.
    /// * `name` - The name of the trace in the legend.
    /// * `data` - The signal values.
    /// * `sample_rate` - The sample rate in Hz for the time-axis.
    pub fn add_signal(&mut self, row: usize, col: usize, name: &str, data: &[f64], sample_rate: f64) {
        let (x_id, y_id) = self.get_axis_ids_in_plotly_format(row, col);

        let time: Vec<f64> = (0..data.len())
            .map(|i| i as f64 / sample_rate)
            .collect();

        let trace = Scatter::new(time, data.to_vec())
            .mode(Mode::Lines)
            .name(name)
            .x_axis(x_id)
            .y_axis(y_id);

        self.plot.add_trace(trace);
    }

    /// Calculates and plots the frequency response (Magnitude) of a real signal.
    /// 
    /// Normalization is performed by dividing the magnitude by the original signal length N.
    /// This plots from 0 Hz to the Nyquist frequency (sample_rate / 2).
    /// * `n_fft` - The size of the FFT. If greater than data length, zeros are padded.
    pub fn add_frequency_response(
        &mut self,
        row: usize,
        col: usize,
        name: &str,
        data: &[f64],
        sample_rate: f64,
        n_fft: usize,
    ) {
        let (x_id, y_id) = self.get_axis_ids_in_plotly_format(row, col);
        let n_original = data.len();
        let n = n_fft.max(n_original);
        
        // 1. Setup FFT Plan
        let mut planner = RealFftPlanner::<f64>::new();
        let fft_planner = planner.plan_fft_forward(n);

        // 2. Prepare Buffers
        let mut indata = fft_planner.make_input_vec();
        // Zero-padding: fill with zeros and copy original data
        for val in indata.iter_mut() { *val = 0.0; }
        indata[..n_original].copy_from_slice(data);
        
        let mut spectrum = fft_planner.make_output_vec();

        // 3. Execute FFT
        fft_planner.process(&mut indata, &mut spectrum).expect("FFT failed");

        // 4. Calculate Magnitude and Normalize by original N
        // Normalizing by n_original ensures the peak matches the sine wave amplitude.
        let magnitudes: Vec<f64> = spectrum
            .iter()
            .map(|&c| c.norm() / n_original as f64)
            .collect();

        // 5. Generate Frequency Axis (0 to fs/2)
        let bin_resolution = sample_rate / n as f64;
        let frequencies: Vec<f64> = (0..magnitudes.len())
            .map(|i| i as f64 * bin_resolution)
            .collect();

        // 6. Plot the Trace
        let trace = Scatter::new(frequencies, magnitudes)
            .mode(Mode::Lines)
            .name(format!("{} (Freq)", name))
            .x_axis(x_id)
            .y_axis(y_id);

        self.plot.add_trace(trace);
    }

    /// Add a raw data series to a specific subplot using (row, col) coordinates.
    pub fn add_raw(&mut self, row: usize, col: usize, name: &str, data: &[f64]) {
        let (x_id, y_id) = self.get_axis_ids_in_plotly_format(row, col);

        let indices: Vec<f64> = (0..data.len())
            .map(|i| i as f64)
            .collect();

        let trace = Scatter::new(indices, data.to_vec())
            .mode(Mode::Lines)
            .name(name)
            .x_axis(x_id)
            .y_axis(y_id);

        self.plot.add_trace(trace);
    }

    /// Show the plot in the default browser.
    pub fn show(&self) {
        self.plot.show();
    }
}
