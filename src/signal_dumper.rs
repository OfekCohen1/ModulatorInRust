//! A utility module for dumping signal data to disk for external analysis.
//! 
//! This module provides a zero-overhead way to save internal signal states 
//! when the `dump-signals` feature is enabled.

/// Dumps a slice of `f64` samples to a binary file in the `dumped_signals/` directory.
/// 
/// The file is written in little-endian binary format, which can be easily 
/// loaded in Python using `np.fromfile(file, dtype=np.float64)`.
/// 
/// This function is only compiled when the `dump-signals` feature is active.
#[cfg(feature = "dump-signals")]
pub fn dump_signal(name: &str, data: &[f64]) {
    use std::io::Write;
    use std::fs::File;
    
    // Ensure the directory exists (though it should have been created manually)
    let path = format!("dumped_signals/{}.bin", name);
    let mut file = File::create(path).expect("Failed to create dump file");
    
    // Write all samples as little-endian bytes
    for &sample in data {
        file.write_all(&sample.to_le_bytes()).expect("Failed to write sample");
    }
}

/// No-op version for production builds without the `dump-signals` feature.
#[cfg(not(feature = "dump-signals"))]
#[inline(always)]
pub fn dump_signal(_name: &str, _data: &[f64]) {
    // Optimized away by the compiler
}
