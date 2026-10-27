//! Shared display metadata for the compute a cloud run executes on.
//!
//! Any UI surface that shows a run's platform to the user — the conversation
//! details panel, the `runner` CLI listing, etc. — should source its labels and
//! icon from here so the surfaces cannot drift.

use std::collections::HashMap;

use warp_graphql::queries::get_runners::{
    Runner, RunnerArch, RunnerConfig, RunnerMacOsVersion, RunnerOs,
};

use crate::ui_components::icons::Icon;











#[cfg(test)]
#[path = "runner_display_tests.rs"]
mod tests;
