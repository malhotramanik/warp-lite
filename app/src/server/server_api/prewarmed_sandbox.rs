//! Client for the pre-warmed sandbox endpoints a standby sandbox uses to fetch its
//! configuration, report preparation state, and prove it is still alive.
//!
//! Every endpoint requires a bearer credential and Namespace workload identity.
// Pre-warmed sandboxes only run in a Namespace container, never in a WASM build.
#![cfg_attr(target_family = "wasm", expect(dead_code))]



use super::ServerApi;
























impl ServerApi {

}



#[cfg(test)]
#[path = "prewarmed_sandbox_tests.rs"]
pub(crate) mod tests;
