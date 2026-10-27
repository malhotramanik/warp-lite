use std::collections::HashMap;

use session_sharing_protocol::common::ParticipantId;
use warpui::{Entity, ModelContext, SingletonEntity};

use crate::ai::ambient_agents::AmbientAgentTaskId;

/// A shared-session-injected prompt queued for a task whose CLI-harness session is registered
/// (see `LocalAgentTaskSyncModel::register_cli_session`) but hasn't started a live PTY yet.
/// File attachments are not supported here — see `PendingCliHarnessPromptQueue::queue`.
///
/// CLI-harness sessions only exist on local/native builds, so this whole type is unread on wasm.
#[cfg_attr(target_family = "wasm", allow(dead_code))]
#[derive(Clone, Debug)]
pub(crate) struct QueuedCliHarnessPrompt {
    pub(crate) prompt: String,
    pub(crate) participant_id: ParticipantId,
}

/// Holds shared-session-injected prompts for third-party-harness ambient tasks whose CLI
/// session has been registered (`LocalAgentTaskSyncModel::register_cli_session`) but has no
/// live PTY yet, so there is nowhere to deliver them. `accept_agent_prompt`
/// (`terminal_view_adaptor.rs`) queues here instead of falling through to the Oz-only
/// `BlocklistAIController` path, which must never create a native `AIConversation` for a
/// task backed by a third-party harness.
///
/// `AgentDriver::subscribe_to_cli_agent_session_events` drains a task's queue directly when its
/// `CLIAgentSessionsModelEvent::Started` fires, delivering each prompt as a genuine PTY
/// follow-up via `TerminalDriver::send_text_to_cli`.
///
/// CLI-harness sessions only exist on local/native builds, so `pending` is unread on wasm.
#[cfg_attr(target_family = "wasm", allow(dead_code))]
#[derive(Default)]
pub(crate) struct PendingCliHarnessPromptQueue {
    pending: HashMap<AmbientAgentTaskId, Vec<QueuedCliHarnessPrompt>>,
}

pub(crate) enum PendingCliHarnessPromptQueueEvent {}

impl PendingCliHarnessPromptQueue {
    pub fn new(_ctx: &mut ModelContext<Self>) -> Self {
        Self::default()
    }

    /// Queues `prompt` for `task_id`'s not-yet-started CLI-harness session.
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub(crate) fn queue(&mut self, task_id: AmbientAgentTaskId, prompt: QueuedCliHarnessPrompt) {
        log::info!(
            "PendingCliHarnessPromptQueue: queuing shared-session prompt for task {task_id} \
             pending CLI-harness session start (participant_id={:?})",
            prompt.participant_id
        );
        self.pending.entry(task_id).or_default().push(prompt);
    }


}

impl Entity for PendingCliHarnessPromptQueue {
    type Event = PendingCliHarnessPromptQueueEvent;
}

impl SingletonEntity for PendingCliHarnessPromptQueue {}

#[cfg(test)]
#[path = "pending_cli_harness_prompt_queue_tests.rs"]
mod tests;
