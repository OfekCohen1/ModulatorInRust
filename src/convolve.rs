//! Discrete Linear Convolution for DSP signals.

/// Standard boundary modes for discrete linear convolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvolveMode {
    /// Return all N + M - 1 samples where signal and kernel have any overlap.
    Full,
    /// Return N samples centered on the original signal's indices.
    /// Offset K = floor((M-1)/2).
    Same,
    /// Return N - M + 1 samples where the kernel completely overlaps the signal.
    Valid,
}

/// A common interface for discrete convolution algorithms.
/// 
/// Enforces the zero-copy mandate by using mutable output slices to avoid
/// heap allocations in DSP hot paths.
pub trait Convolver {
    /// Computes the discrete linear convolution (f * g)[n] = sum(f[n-m] * g[m]).
    /// 
    /// # Arguments
    /// * `signal` - The input signal f[n] of length N.
    /// * `kernel` - The filter kernel g[n] of length M.
    /// * `output` - Pre-allocated buffer for the result.
    /// * `mode` - Boundary handling mode (Full, Same, or Valid).
    fn convolve(&self, signal: &[f64], kernel: &[f64], output: &mut [f64], mode: ConvolveMode);
}

/// A direct time-domain implementation of the sliding sum convolution.
///
/// This implementation pre-flips the kernel and uses an optimized sliding dot
/// product with functional iterators to enable SIMD auto-vectorization.
pub struct DirectConvolver;

impl Convolver for DirectConvolver {
    fn convolve(&self, signal: &[f64], kernel: &[f64], output: &mut [f64], mode: ConvolveMode) {
        if signal.is_empty() || kernel.is_empty() {
            return;
        }

        let n = signal.len();
        let m = kernel.len();

        let start_offset = match mode {
            ConvolveMode::Full => 0,
            ConvolveMode::Same => (m - 1) / 2,
            ConvolveMode::Valid => m - 1,
        };

        // The kernel is flipped in the sliding sum definition:
        // (f * g)[n] = sum_{i=0}^{M-1} f[n-i] * g[i]
        // We pre-flip the kernel once to use it as a simple dot-product.
        let mut flipped_kernel = kernel.to_vec();
        flipped_kernel.reverse();

        for j in 0..output.len() {
            let full_idx = j + start_offset;
            
            // Calculate the valid range of the flipped kernel that overlaps with the signal.
            // For a given full_idx (n in the formula), the signal indices are [n-m+1, n].
            // We want to find the intersection of [n-m+1, n] and [0, n_sig-1].
            
            let sig_end = full_idx as isize;
            let sig_start = sig_end - (m as isize) + 1;
            
            let overlap_start = sig_start.max(0);
            let overlap_end = sig_end.min((n as isize) - 1);
            
            if overlap_start <= overlap_end {
                // Map signal overlap back to flipped kernel indices.
                // Flipped kernel represents [g[m-1], g[m-2], ..., g[0]]
                // These correspond to signal indices [n-m+1, n-m+2, ..., n]
                let k_start = (overlap_start - sig_start) as usize;
                let k_end = (overlap_end - sig_start) as usize;
                
                let signal_slice = &signal[overlap_start as usize ..= overlap_end as usize];
                let kernel_slice = &flipped_kernel[k_start ..= k_end];
                
                output[j] = signal_slice.iter()
                    .zip(kernel_slice.iter())
                    .map(|(&s, &k)| s * k)
                    .sum();
            } else {
                output[j] = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn convolver() -> DirectConvolver {
        DirectConvolver
    }

    #[rstest]
    #[case(ConvolveMode::Full, vec![1.0, 1.0], vec![1.0, 1.0], vec![1.0, 2.0, 1.0])]
    #[case(ConvolveMode::Same, vec![1.0, 1.0, 1.0], vec![1.0, 2.0, 3.0], vec![3.0, 6.0, 5.0])] // Center K=1. Full is [1, 3, 6, 5, 3]
    #[case(ConvolveMode::Valid, vec![1.0, 2.0, 3.0, 4.0], vec![1.0, 1.0], vec![3.0, 5.0, 7.0])]
    fn test_convolution_deterministic(
        convolver: DirectConvolver,
        #[case] mode: ConvolveMode,
        #[case] signal: Vec<f64>,
        #[case] kernel: Vec<f64>,
        #[case] expected: Vec<f64>,
    ) {
        // --- Arrange ---
        let mut output = vec![0.0; expected.len()];
        let epsilon = 1e-10;

        // --- Act ---
        convolver.convolve(&signal, &kernel, &mut output, mode);
        
        // --- Assert ---
        assert!(
            output.iter()
                .zip(expected.iter())
                .all(|(a, b)| (a - b).abs() < epsilon),
            "Output {:?} does not match expected {:?}", output, expected
        );
    }

    #[rstest]
    fn test_convolution_unit_impulse(convolver: DirectConvolver) {
        // --- Arrange ---
        let signal = vec![1.0, 0.0, 0.0, 0.0];
        let kernel = vec![0.25, 0.5, 0.25];
        let mut output = vec![0.0; 6];
        let expected = vec![0.25, 0.5, 0.25, 0.0, 0.0, 0.0];
        
        // --- Act ---
        convolver.convolve(&signal, &kernel, &mut output, ConvolveMode::Full);
        
        // --- Assert ---
        assert!(
            output.iter()
                .zip(expected.iter())
                .all(|(a, b)| (a - b).abs() < f64::EPSILON)
        );
    }
}
