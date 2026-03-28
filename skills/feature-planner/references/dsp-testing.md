# DSP Testing Best Practices

When writing tests for signal processing features in Rust, follow these patterns to ensure correctness and maintainability.

## 1. Floating-Point Comparisons
Floating-point results are almost never exact. Never use `assert_eq!` for `f64`. Instead, use a small `EPSILON` (e.g., `1e-10`) to check for closeness.

```rust
const EPSILON: f64 = 1e-10;
assert!((actual - expected).abs() < EPSILON);
```

## 2. Parameterized Tests with `rstest`
Use `rstest` to test multiple scenarios with a single piece of test logic. This is ideal for verifying that a modulator works across different frequencies or indices.

```rust
#[rstest]
#[case(100.0, 1.0)]  // 100Hz, Index 1.0
#[case(440.0, 0.5)]  // 440Hz, Index 0.5
fn test_algorithm(#[case] frequency: f64, #[case] index: f64) {
    // Test logic here
}
```

## 3. Reference Vector Generation
If a signal has a known mathematical formula, generate the "expected" vector inside the test using the same parameters. This makes the test self-documenting.

```rust
let expected_signal: Vec<f64> = (0..num_samples)
    .map(|i| {
        let t = i as f64 / sample_rate;
        // Formula here
    })
    .collect();
```

## 4. Edge Case Testing
Always consider these scenarios in your `rstest` cases:
- **Zero Input**: What happens if the message is completely silent (all zeros)?
- **Max Input**: What happens at the 1.0 or -1.0 boundaries?
- **Oversized Input**: Does the safety/normalization logic trigger correctly?
- **High Frequency**: Does the algorithm behave correctly near the Nyquist frequency?
