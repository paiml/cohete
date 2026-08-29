//! Cohete: nightly E2E verification for the sovereign AI stack on Jetson.
//!
//! WHY THIS FILE EXISTS AT ALL.
//!
//! cohete was bin-only: every module was declared with `mod` in `main.rs` and
//! there was no library target. `cargo test --lib` on such a crate is not an
//! empty run, it is a HARD ERROR:
//!
//! ```text
//! error: no library targets found in package `cohete`
//! ```
//!
//! The shared `sovereign-ci.yml` runs exactly that, so `ci / test` could never
//! pass, so `ci / gate` could never pass — cohete's PRs were unmergeable for a
//! reason no contributor could see in their own diff. The pre-push hook ran the
//! same command locally, which is why every push here used `--no-verify` and
//! why 84 rustfmt diffs and two clippy errors accumulated behind it.
//!
//! A gate that CANNOT be satisfied does not raise the bar; it teaches everyone
//! to go around it, and then the real findings arrive through the same hole.
//!
//! So the modules move into a library and `main.rs` becomes a thin binary over
//! it. That is not a workaround for the gate — it is what the gate was asking
//! for. These tiers are ordinary testable code: `runner` shells out and parses,
//! `types` is the artifact schema, and each `tierN_*` is a pure-ish pipeline
//! from command output to a verdict. None of it needed to be locked inside a
//! binary, and locking it there is why the crate has almost no unit tests.

// `must_use_candidate` is a PEDANTIC lint and it fires 22 times here purely
// because this change made the modules public — the functions did not become
// more dangerous to ignore, they became visible. Annotating 22 signatures with
// `#[must_use]` would be noise standing in for a review nobody asked for, and
// none of these returns is one you can drop by accident: every caller in the
// tiers immediately reads `.exit_code` or `.stdout`.
//
// Allowed at the crate root, with the reason, rather than silenced per call
// site — a scattered `#[allow]` is how a lint stops being a decision.
#![allow(clippy::must_use_candidate)]

pub mod runner;
pub mod tier1_smoke;
pub mod tier2_hardware;
pub mod tier3_functional;
pub mod tier4_integration;
pub mod tier5_performance;
pub mod types;
pub mod uat;
