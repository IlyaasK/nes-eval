//! Library surface for the grader crate.
//!
//! Exposes the wasmtime + fuel + import-stub wiring used by the grader
//! binary so other tools can reuse `WasmCandidate` without duplicating it.

pub mod corpus;
pub mod ref_cache;
pub mod wasm_candidate;
