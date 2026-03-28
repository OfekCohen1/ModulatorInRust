---
name: feature-planner
description: A Test-Driven Development (TDD) and Design-First workflow for implementing new features. Use when a user asks to develop or implement a specific feature or algorithm.
---

# Feature Planner Skill

This skill provides a structured, multi-step workflow for implementing features correctly using a TDD approach. It ensures that designs are clear, tests are well-defined, and implementations are verified before finality.

## The Feature-Planning Workflow

Follow these steps in order for every new feature request.

### 1. Design & Skeleton
When a user requests a feature (e.g., "Implement FM Modulation"), do NOT implement it immediately.
1.  **Analyze**: Understand the mathematical or logic requirements.
2. **Plan and Design**: Ask questions to create a plan. Ask questions related to design, implementation details, frameworks, mathematical ways to implement, etc.  
3. **Propose Skeleton**: When done planning, create a new module or update an existing one with **only** the trait/struct definitions and empty function signatures (using `todo!()`).
4. **Refine**: Explain the proposed interface to the user.
5. Ask for confirmation about the design and skeleton before proceeding to the tests.

### 2. Test-First Definition
Before writing implementation logic, you MUST define the success criteria.
1.  **Write Failing Tests**: Use `rstest` to define the tests that the new feature must pass.
2. **Suggest Tests**: Suggest tests based on your understanding of the feature. Ask the user if your suggestions suffice, and query him for more types of tests.  
3.  **Mandatory Question**: You MUST stop and ask the user at least **one specific question** about the tests. Examples:
    - "What is the expected tolerance (EPSILON) for this calculation?"
    - "Should we test for negative input values or only positive?"
    - "Do you have a specific reference vector we should use to verify the output?"
4.  **Wait for Confirmation**: Do not proceed to implementation until the user has refined or approved the test cases. Specifically, after implementing the tests, ask if anything else should be changed in the tests.

### 3. Implementation (Skeleton Filling)
Once tests are approved, implement the feature.
1. **Design the Implementation**: Ask questions about the implementation, both design and logic. Only once design and logic are finished, start implementing. 
2. **Minimalist Implementation**: Write just enough code to make the tests pass.
2.  **Rust Idioms**: Ensure the code follows the standards in the workspace (e.g., `gemini.md` styling, descriptive names).
3.  **Expert Tips**: Provide 1-2 tips about the implementation, that are specific for rust(e.g., numerical stability, performance).

### 4. Verification & Iteration
1.  **Run Tests**: Execute `cargo test <feature_module>`.
2.  **Diagnose Failures**: If tests fail, analyze the output and fix the code. Repeat until all tests pass.
3.  **Refactor**: Once green, look for opportunities to simplify the logic or improve performance without changing the behavior.

## Best Practices for DSP Features

For signal processing features, consult [references/dsp-testing.md](references/dsp-testing.md) for patterns on testing floating-point signals and generating test vectors.

## Proactive Communication
Always ask for more details about the **test, design, or implementation** if any requirement is ambiguous. Never "guess" a complex algorithm's behavior without confirming with the user.
