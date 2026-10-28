//! This module contains all auxiliary code for hitting the GenerateAIInputSuggestions endpoint in
//! warp-server that serves Agent Predict.

mod api;


pub use api::*;
use chrono::NaiveDateTime;
use itertools::Itertools;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    pub pwd: Option<String>,
    pub git_branch: Option<String>,
    pub exit_code: i64,
}
/// Used for AI-powered input suggestions (next action prediction). We
/// pass relevant context per block to AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessageInput {
    pub input: String,
    pub output: String,
    pub context: CommandContext,
}

/// A section of command history that can be used as context for intelligent autosuggestions.
/// The context includes a consecutive sequence of commands in a session (previous_commands) and the next_command,
/// previous_commands can be similar to the user's recently run commands, and next_command can be used
/// to predict the next command.
#[derive(Clone)]
pub struct HistoryContext {
    pub previous_commands: Vec<crate::persistence::model::Command>,
    pub next_command: crate::persistence::model::Command,
}

/// Context needed for a Next Command suggestion that remains constant regardless of what
/// the user has typed in the input. This is reused as the user edits the input and generates new suggestions.
#[derive(Clone)]
pub struct NextCommandContext {
    pub history_contexts: Vec<HistoryContext>,
    pub context_messages: Vec<ContextMessageInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandWithContext {
    pub command: String,
    pub pwd: Option<String>,
    pub exit_code: Option<i32>,
    pub git_branch: Option<String>,
    pub start_ts: Option<NaiveDateTime>,
}

impl From<&crate::persistence::model::Command> for CommandWithContext {
    fn from(command: &crate::persistence::model::Command) -> Self {
        CommandWithContext {
            command: command.command.clone(),
            pwd: command.pwd.clone(),
            exit_code: command.exit_code,
            git_branch: command.git_branch.clone(),
            start_ts: command.start_ts,
        }
    }
}

impl HistoryContext {
    pub fn to_context_string(&self) -> String {
        self.previous_commands
            .iter()
            .chain(std::iter::once(&self.next_command))
            .map(CommandWithContext::from)
            .filter_map(|command_with_context| serde_json::to_string(&command_with_context).ok())
            .join("\n")
    }

    // Same as to_context_string, but skips any commands up to and including the specified ID.
    // Returns a tuple of the resulting string and a boolean indicating whether any commands were skipped.
}





#[cfg(test)]
#[path = "generate_ai_input_suggestions_tests.rs"]
mod tests;
