# AI Agent Guidelines for Rust Development

This document outlines the best practices and organizational principles for AI agents working on this Rust project.

## Rust Best Practices

*   **Ownership and Borrowing:** Prioritize idiomatic ownership and borrowing. Prefer immutable references (`&T`) over mutable ones (`&mut T`) unless mutation is necessary. Avoid unnecessary allocations and cloning.
*   **Error Handling:** Use `Result` and `Option` for error handling. Avoid `unwrap()` and `expect()` in production code unless it's a proven invariant. Leverage the `?` operator for clean error propagation. Use the `thiserror` crate for defining custom error types.
*   **Audio Processing:** Use the `rodio` crate for all audio playback and processing requirements.
*   **Type Safety:** Use Rust's strong type system to your advantage. Create custom types and enums to represent domain logic, making invalid states unrepresentable.
*   **Clippy and Formatting:** All code must be formatted using `rustfmt` and should pass all `clippy` lints without warnings.
*   **Modern Rust:** Target the latest stable Rust edition (currently 2024 as per `Cargo.toml`).

## Clean Organization

*   **Module Structure:** Maintain a clear and hierarchical module structure. Use `mod.rs` or the newer directory-based module system appropriately. Keep modules focused and cohesive.
*   **Separation of Concerns:** Separate business logic from infrastructure (I/O, database, API) using traits and dependency injection where applicable.
*   **Visibility:** Use the most restrictive visibility possible (e.g., `pub(crate)` or private by default) to encapsulate implementation details.

## Code and TDD Driven Style

*   **Test-Driven Development (TDD):** Follow a TDD workflow. Write failing tests before implementing the logic.
*   **Unit Tests:** Place unit tests in the same file as the code they test, using a `cfg(test)` module.
*   **Integration Tests:** Place integration tests in the `tests/` directory at the project root.
*   **Documentation Tests:** Write doc-tests for public APIs to ensure examples remain valid and the API is well-documented.
*   **Mocking:** Use traits to enable easy mocking of external dependencies during testing.

## Mandatory Documentation of Changes

*   **STEPS.md:** Every AI-driven change, including refactorings, feature additions, and bug fixes, must be recorded and explained in the `STEPS.md` file.
*   **Content of STEPS.md:** Each entry should include:
    *   The goal of the change.
    *   The approach taken.
    *   Key decisions made.
    *   Verification steps performed.

## Architecture
* The architecture for this project is described in the ARCHITECTURE.md file.

## Additional Guidelines
*   **Version Control:** Use Git for version control.
*   **Continuous Integration:** Use GitHub Actions for continuous integration.
*   **Code Review:** All code changes must be reviewed by at least one other developer.
*   **Code Style:** Follow the Rust Style Guide.
*   **Documentation:** Write comprehensive documentation for all public APIs.
*   **Testing:** Write comprehensive tests for all code.
*   **Continuous Deployment:** Deploy changes automatically to production.

## Pre-approved dependencies
*   **rodio:** For audio playback and processing.
*   **thiserror:** For defining custom error types.
*   **midir:** For real-time MIDI input/output, including virtual MIDI ports.
*   **crossbeam-channel:** For thread-safe communication between the MIDI/keyboard and audio threads.
*   **rdev:** For OS-level keyboard press/release event capture (computer-keyboard MIDI controller).
