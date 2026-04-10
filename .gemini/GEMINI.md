# Agent Guidelines for Rust Code Quality

This document provides guidelines for maintaining high-quality Rust code. These rules MUST be followed by all AI coding agents and contributors.


## STRICT TERMINAL FORMATTING MANDATE (NO LATEX)
**CRITICAL SYSTEM LIMITATION:** You are outputting text directly to a raw terminal console that DOES NOT support LaTeX rendering. 
* **NEVER** use `$` or `$$` delimiters for inline or display math.
* **NEVER** use LaTeX commands like `\frac`, `\sum`, `\pi`, or `\omega`.
* **ALWAYS** write mathematics, equations, and variables using strictly plain ASCII text. 
* *Examples of acceptable ASCII math:* `x^2 + y^2 = z^2`, `theta`, `pi`, `sum(x_i for i=0 to N)`, `H(e^(jw))`.
* If you output LaTeX, the user's console output will break and become unreadable. 

## Project Context
This repository contains natively written Rust algorithms for digital communication systems and digital signal processing (DSP). The focus is on implementing high-level algorithmic theory efficiently in Rust, emphasizing pure software performance and mathematical correctness.

## Global Coding Standards (Zero-Cost Rust)
All AI agents operating in this repository MUST adhere to the following standards, regardless of their specific skill role:
* **Style:** Strict adherence to the official Rust Style Guide (https://doc.rust-lang.org/style-guide/).
* **Memory Pipeline:** Zero-copy by default. Heap allocations (`Vec`, `Box`, `Arc`) are strictly forbidden in DSP hot paths. Data must flow via slices (`&[T]`, `&mut [T]`) and `const` generic sized arrays.
* **Math Safety:** Standard operators (`+`, `-`, `*`) are dangerous at physical signal boundaries. You must explicitly define overflow behavior using `.wrapping_add()`, `.saturating_add()`, etc., where appropriate.
* **Auto-Vectorization:** LLVM must be able to vectorize DSP loops. Favor functional iterators (`.zip()`, `.fold()`, `.chunks_exact()`) over manual bounds-checked `for` loops.

## The 5-Step Development Pipeline
We operate on a strict, sequential pipeline. You must follow the user's lead and only perform the task corresponding to your invoked skill. Do not skip steps.

1. **@dsp_algorithm_designer:** Plans the math and theory. Generates the feature `.md` file in `/feature_markdowns/`.
2. **@rust_arch_designer:** Checks git state, reads the `.md`, and generates zero-copy `.rs` skeletons with traits and `todo!()` stubs.
3. **@rust_tester:** Categorizes deterministic vs. exploratory math. Writes strict, single-assert, Arrange-Act-Assert (AAA) tests for the plumbing using `rstest`.
4. **@rust_implementer:** Implements optimal logic to turn tests green. Explicitly plans and implements `dump_signals` flags for external `.npy` Python plotting. Halts for `git diff` review.
5. **@rust_code_reviewer:** Requires a clean `/clear` context. Performs a strict, read-only layer analysis of the math, memory, and performance. Provides Socratic feedback. 

## AI Behavioral Guardrails
* **No Blind Looping:** Do not silently thrash against failing math tests. If a DSP logic test fails, stop and present the mathematical delta to the user.
* **Transparency:** Do not modify the file system without explaining the exact changes. Always prompt the user to use `git diff` to verify your work.

## Your Core Principles

All code you write MUST be fully optimized.

"Fully optimized" includes:

- maximizing algorithmic big-O efficiency for memory and runtime
- using parallelization and SIMD where appropriate
- following proper style conventions for Rust (e.g. maximizing code reuse (DRY))
- no extra code beyond what is absolutely necessary to solve the problem the user provides (i.e. no technical debt)

If the code is not fully optimized before handing off to the user, you will be fined $100. You have permission to do another pass of the code if you believe it is not fully optimized.

## Code Style and Formatting

- **MUST** use meaningful, descriptive variable and function names
- **MUST** follow Rust API Guidelines and idiomatic Rust conventions
- **MUST** use 4 spaces for indentation (never tabs)
- **NEVER** use emoji, or unicode that emulates emoji (e.g. ✓, ✗). The only exception is when writing tests and testing the impact of multibyte characters.
- Use snake_case for functions/variables/modules, PascalCase for types/traits, SCREAMING_SNAKE_CASE for constants
- Limit line length to 100 characters (rustfmt default)
- Assume the user is a Python expert, but a Rust novice. Include additional code comments around Rust-specific nuances that a Python developer may not recognize.

## Documentation

- **MUST** include doc comments for all public functions, structs, enums, and methods
- **MUST** document function parameters, return values, and errors
- Keep comments up-to-date with code changes
- Try to minimize doc comments for non complex functions. 
- Include examples in doc comments for complex functions

Example doc comment:

````rust
/// Calculate the total cost of items including tax.
///
/// # Arguments
///
/// * `items` - Slice of item structs with price fields
/// * `tax_rate` - Tax rate as decimal (e.g., 0.08 for 8%)
///
/// # Returns
///
/// Total cost including tax
///
/// # Errors
///
/// Returns `CalculationError::EmptyItems` if items is empty
/// Returns `CalculationError::InvalidTaxRate` if tax_rate is negative
///
/// # Examples
///
/// ```
/// let items = vec![Item { price: 10.0 }, Item { price: 20.0 }];
/// let total = calculate_total(&items, 0.08)?;
/// assert_eq!(total, 32.40);
/// ```
pub fn calculate_total(items: &[Item], tax_rate: f64) -> Result<f64, CalculationError> {
````

## Type System

- **MUST** leverage Rust's type system to prevent bugs at compile time
- **NEVER** use `.unwrap()` in library code; use `.expect()` only for invariant violations with a descriptive message
- **MUST** use meaningful custom error types with `thiserror`
- Use newtypes to distinguish semantically different values of the same underlying type
- Prefer `Option<T>` over sentinel values

## Error Handling

- **NEVER** use `.unwrap()` in production code paths
- **MUST** use `Result<T, E>` for fallible operations
- **MUST** use `thiserror` for defining error types and `anyhow` for application-level errors
- **MUST** propagate errors with `?` operator where appropriate
- Provide meaningful error messages with context using `.context()` from `anyhow`

## Function Design

- **MUST** keep functions focused on a single responsibility
- **MUST** prefer borrowing (`&T`, `&mut T`) over ownership when possible
- Limit function parameters to 5 or fewer; use a config struct for more
- Return early to reduce nesting
- Use iterators and combinators over explicit loops where clearer

## Struct and Enum Design

- **MUST** keep types focused on a single responsibility
- **MUST** derive common traits: `Debug`, `Clone`, `PartialEq` where appropriate
- Use `#[derive(Default)]` when a sensible default exists
- Prefer composition over inheritance-like patterns
- Use builder pattern for complex struct construction
- Make fields private by default; provide accessor methods when needed

## Testing

- **MUST** write unit tests for all new functions and types
- **MUST** mock external dependencies (APIs, databases, file systems)
- Follow the Arrange-Act-Assert pattern
- Do not commit commented-out tests
- Use the `rstest` library for testing.

## Imports and Dependencies

- **MUST** avoid wildcard imports (`use module::*`) except for preludes, test modules (`use super::*`), and prelude re-exports
- **MUST** document dependencies in `Cargo.toml` with version constraints
- Use `cargo` for dependency management
- Organize imports: standard library, external crates, local modules
- Use `rustfmt` to automate import formatting
- For python scripts, use `uv` package manager

## Rust Best Practices

- **NEVER** use `unsafe` unless absolutely necessary; document safety invariants when used
- **MUST** call `.clone()` explicitly on non-`Copy` types; avoid hidden clones in closures and iterators
- **MUST** use pattern matching exhaustively; avoid catch-all `_` patterns when possible
- **MUST** use `format!` macro for string formatting
- Use iterators and iterator adapters over manual loops
- Use `enumerate()` instead of manual counter variables
- Prefer `if let` and `while let` for single-pattern matching

## Memory and Performance

- **MUST** avoid unnecessary allocations; prefer `&str` over `String` when possible
- **MUST** use `Cow<'_, str>` when ownership is conditionally needed
- Use `Vec::with_capacity()` when the size is known
- Prefer stack allocation over heap when appropriate
- Use `Arc` and `Rc` judiciously; prefer borrowing

**Remember:** Prioritize clarity and maintainability over cleverness.