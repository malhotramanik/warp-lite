use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use warp_core::command::ExitCode;

use crate::terminal::model::block::BlockId;
use crate::terminal::model::terminal_model::BlockIndex;

/// Contains context about a completed terminal command block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockContext {
    /// The ID of the block whose contents are included in the query.
    ///
    /// This is not actually included in the query payload but is used for tracing blocks
    /// passed as context in conversation history, which may be useful for deduping instances
    /// of passing the same block as context or for rendering UI.
    #[serde(rename = "block_id")]
    pub id: BlockId,
    /// The index into the blocklist where this block is located.
    pub index: BlockIndex,
    pub command: String,
    pub output: String,
    pub exit_code: ExitCode,
    /// Whether this block was auto-attached (via AgentViewBlockContext feature)
    /// rather than manually attached by the user.
    #[serde(default)]
    pub is_auto_attached: bool,
    /// Timestamp when the command started executing.
    #[serde(default)]
    pub started_ts: Option<DateTime<Local>>,
    /// Timestamp when the command finished executing.
    #[serde(default)]
    pub finished_ts: Option<DateTime<Local>>,

    // Environment fields — populated by the constructors below, left as
    // None at construction sites that don't need them.
    /// The working directory where the command was executed.
    #[serde(default)]
    pub pwd: Option<String>,
    /// The shell type (e.g., "zsh", "bash").
    #[serde(default)]
    pub shell: Option<String>,
    /// The username of the user who executed the command.
    #[serde(default)]
    pub username: Option<String>,
    /// The hostname of the machine where the command was executed.
    #[serde(default)]
    pub hostname: Option<String>,
    /// The git branch at the time of execution.
    #[serde(default)]
    pub git_branch: Option<String>,
    /// The operating system name (e.g., "MacOS", "Linux").
    #[serde(default)]
    pub os: Option<String>,
    /// The terminal session ID.
    #[serde(default)]
    pub session_id: Option<u64>,
}

impl BlockContext {
}

#[cfg(test)]
#[path = "block_context_tests.rs"]
mod tests;
