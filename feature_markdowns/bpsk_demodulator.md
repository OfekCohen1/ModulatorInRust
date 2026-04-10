# BPSK Demodulator: Algorithmic Specification

This document defines the mathematical and theoretical foundation for a Binary Phase Shift Keying (BPSK) demodulator. The design assumes a coherent receiver with perfect carrier and symbol timing synchronization.

## 1. Mathematical Signal Model

The received BPSK signal $r(t)$ is represented as:

$$r(t) = A \cdot d(t) \cdot \cos(2\pi f_c t + \phi) + n(t)$$

Where:
- $A$: Signal amplitude.
- $d(t)$: The baseband bipolar NRZ signal ($bit '1' \rightarrow +1, bit '0' \rightarrow -1$).
- $f_c$: Carrier frequency.
- $\phi$: Initial phase (assumed $0$ for perfect synchronization).
- $n(t)$: Additive White Gaussian Noise (AWGN).

## 2. Demodulation Algorithm

### Step 1: Coherent Downconversion (Product Modulator)
The received signal is multiplied by a local carrier reference:

$$x(t) = r(t) \cdot \cos(2\pi f_c t)$$
$$x(t) = A \cdot d(t) \cdot \cos^2(2\pi f_c t)$$
$$x(t) = \frac{A}{2}d(t) [1 + \cos(4\pi f_c t)]$$

This results in a baseband component $\frac{A}{2}d(t)$ and a double-frequency component at $2f_c$.

### Step 2: Low-Pass Filtering (LPF)
To isolate the baseband signal and suppress the $2f_c$ term, a Finite Impulse Response (FIR) filter is employed.

- **Filter Type:** Windowed Sinc.
- **Window Function:** Hamming window ($w(n) = 0.54 - 0.46 \cos(2\pi n / M)$).
- **Cutoff Frequency ($f_{cutoff}$):** $2 \cdot R_s$, where $R_s$ is the symbol rate.
- **Impulse Response ($h(n)$):**
  $$h_{ideal}(n) = 2 f_{norm} \cdot \text{sinc}(2 f_{norm} (n - M/2))$$
  $$h(n) = h_{ideal}(n) \cdot w(n)$$
  Where $f_{norm} = f_{cutoff} / f_{sample}$.

### Step 3: Matched Filtering via Convolution
The optimal receiver for a signal in AWGN utilizes a Matched Filter. The impulse response of the matched filter $h(n)$ is the time-reversed version of the transmitted pulse shape $p(n)$:

$$h(n) = p(M - 1 - n)$$

The filtered baseband signal $z(n)$ is computed via discrete linear convolution:

$$z(n) = \sum_{k=0}^{N-1} y(k) \cdot h(n - k)$$

In this implementation, the convolution is performed using the "Same" boundary mode. This ensures that the output signal maintains the same length as the input and that the filter's group delay is handled consistently with the modulator.

### Step 4: Decision Logic
The decision on the $k$-th bit is made by sampling the matched filter output at the "middle of the eye," where the signal-to-noise ratio is maximized. 

Due to the use of "Same" mode convolution and the centering of symbols during upsampling, the optimal sampling instant $n_k$ for the $k$-th symbol is:

$$n_k = k \cdot S + \lfloor \frac{M - 1}{2} \rfloor$$

Where:
- $k$: The symbol index ($0, 1, 2, \dots$).
- $S$: Samples per symbol.
- $M$: Length of the pulse-shaping/matched filter.

The final bit decision is made using a zero-threshold:

$$\hat{b}_k = \begin{cases} 1 & \text{if } z(n_k) > 0 \\ 0 & \text{if } z(n_k) \le 0 \end{cases}$$

## 3. Theoretical Constraints
- The carrier frequency $f_c$ must be significantly higher than the symbol rate $R_s$ to ensure spectral separation between the baseband and double-frequency components.
- The sample rate $f_s$ must satisfy the Nyquist criterion for the passband signal ($f_s > 2(f_c + R_s)$).
