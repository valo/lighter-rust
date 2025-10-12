### Plan to Ensure the Signing Module Works and Align It with the Python SDK

1. **Baseline Assessment**
   * Audit the existing `signers` module (Rust) to understand current functionality, public API surface, and integration points (e.g., `LighterClient`). 
   * Identify all call sites and tests (unit/integration) that currently rely on signing logic to determine impact and coverage gaps.

2. **Cross-SDK Comparison**
   * Review the signing implementation in the Python SDK (`lighter-python`) to document key behaviors: supported signer types, message hashing format, deterministic signatures, error-handling strategy, and API ergonomics.
   * Highlight any divergences between Rust and Python implementations (e.g., naming conventions, abstractions, FFI responsibilities, feature coverage).

3. **Define Target Behavior & Interfaces**
   * Establish a contract for the Rust signing module that mirrors the Python SDK, including the data required to sign each transaction type and expected outputs.
   * Decide which abstractions are redundant in Rust (e.g., thin wrappers around FFI calls) and outline how to simplify them while keeping parity with Python’s public API.

4. **Refine Implementation**
   * Refactor the Rust signer modules to remove unnecessary layers, aligning structure and naming with the Python SDK for clarity (e.g., consolidate `Signer` trait usage, reconsider module boundaries, simplify FFI struct).
   * Ensure consistent error propagation using `LighterError::Signing`, adding context where needed to match Python’s diagnostics.

5. **Strengthen Test Coverage**
   * Port signing test vectors from the Python SDK (if available) or generate new deterministic fixtures to validate each signer method (`sign_message`, order/transfer signing, etc.).
   * Add unit tests for mnemonic/HD wallet derivation, address retrieval, and FFI boundary conditions (mocking library interactions where possible).

6. **Validate FFI Integration**
   * Verify dynamic library discovery works across target platforms; add fallbacks or configuration options as necessary.
   * Introduce integration tests (feature-gated if needed) that exercise the FFI signer using real or mocked shared libraries, ensuring parity with Python behavior.

7. **Documentation & Examples**
   * Update README/examples to reflect simplified signer usage, aligning code snippets with Python SDK patterns.
   * Document platform requirements, error messages, and troubleshooting steps for FFI signer setup.

8. **Regression & Release Prep**
   * Run the full test suite (`cargo test`) and clippy/format checks to ensure refactors don’t introduce regressions.
   * Plan for semantic versioning impact, noting changes in public API or dependency requirements.
