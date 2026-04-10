import numpy as np
import matplotlib.pyplot as plt
import os

def plot_signal_and_fft(filename, title, sample_rate=1000.0):
    """
    Loads a raw binary f64 signal and plots its Time and Frequency domain representations.
    """
    filepath = os.path.join("dumped_signals", f"{filename}.bin")
    if not os.path.exists(filepath):
        print(f"Signal file not found: {filepath}")
        return

    # Load binary data (f64 little-endian)
    data = np.fromfile(filepath, dtype=np.float64)
    n = len(data)
    t = np.arange(n) / sample_rate

    # Calculate FFT
    fft_data = np.fft.fft(data)
    fft_freq = np.fft.fftfreq(n, 1/sample_rate)
    
    # Only take positive frequencies for plotting
    pos_idx = fft_freq >= 0
    fft_freq = fft_freq[pos_idx]
    fft_mag = np.abs(fft_data[pos_idx]) / n

    # Create 2x1 Grid
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8))
    fig.suptitle(f"Diagnostic: {title}")

    # Time Domain Plot
    ax1.plot(t, data, color='tab:blue')
    ax1.set_title("Time Domain")
    ax1.set_xlabel("Time (s)")
    ax1.set_ylabel("Amplitude")
    ax1.grid(True, linestyle='--', alpha=0.7)

    # Frequency Domain Plot
    ax2.plot(fft_freq, fft_mag, color='tab:red')
    ax2.set_title("Frequency Domain (Magnitude)")
    ax2.set_xlabel("Frequency (Hz)")
    ax2.set_ylabel("Magnitude")
    ax2.grid(True, linestyle='--', alpha=0.7)

    plt.tight_layout()

if __name__ == "__main__":
    print("--- Starting Modem Signal Analysis ---")
    
    # 1. Modulator Stages
    plot_signal_and_fft("original_message", "Original Message (Modulator Input)")
    plot_signal_and_fft("am_modulated_signal", "AM Modulated Signal")

    # 2. Demodulator Diagnostics
    plot_signal_and_fft("lpf_impulse_response", "LPF Characterization (Impulse Response)")
    plot_signal_and_fft("am_after_mixer", "Coherent Demod: Post-Mixer Signal")
    plot_signal_and_fft("am_after_lpf_final", "Coherent Demod: Recovered Message (Post-LPF)")
    
    # 3. Final Comparison
    plot_signal_and_fft("recovered_message", "Final Recovered Signal (Main Entry)")

    plt.show()
    
    print("--- Analysis Complete ---")
