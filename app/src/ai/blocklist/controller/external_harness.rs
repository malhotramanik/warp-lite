// Plumbing for harnesses that run outside Warp's agent loop but render through a native
// conversation: the harness translates its own protocol into MAA response events, which flow
// through the same controller path as events from the MAA server. The conversation stays owned
// by this client (so task status syncing keeps working); only the turn itself is the harness's.
use session_sharing_protocol::common::{AgentAttachment, ParticipantId};
use warpui::{ModelContext, SingletonEntity};

use super::BlocklistAIController;
use crate::ai::agent::conversation::{AIConversationId, ConversationDriver};
use crate::ai::agent::BaseUserQuery;
use crate::ai::blocklist::history_model::BlocklistAIHistoryModel;


/// A prompt relayed to the external harness driving a native conversation.
#[derive(Clone, Debug)]
pub(in crate::ai) struct ExternalHarnessPrompt {
    pub text: String,
    /// Context the prompt's author attached, kept in its shared-session wire shape so the
    /// harness can decide how to present each kind to its agent.
    pub attachments: Vec<AgentAttachment>,
    /// The shared-session participant who injected the prompt, so the turn it opens is
    /// attributed to them for viewers.
    pub participant_id: ParticipantId,
    /// The `Request.Input.UserQuery` warp-server injected with the prompt, when it sent one.
    /// Its attribution is copied onto the user query echoed into the conversation.
    pub base: Option<BaseUserQuery>,
}

/// A request to interrupt the turn an external harness is running, relayed to that harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExternalHarnessCancel;

impl BlocklistAIController {



    /// Whether a running external harness owns `conversation_id`'s turns, so a cancellation
    /// must go to it rather than to the terminal.
    fn is_cancelled_via_external_harness(
        &self,
        conversation_id: AIConversationId,
        ctx: &ModelContext<Self>,
    ) -> bool {
        self.native_prompt_conversation_id == Some(conversation_id)
            && self
                .external_harness_cancel_sink
                .as_ref()
                .is_some_and(|sink| !sink.is_closed())
            && BlocklistAIHistoryModel::as_ref(ctx)
                .conversation(&conversation_id)
                .is_some_and(|conversation| {
                    conversation.driver() == ConversationDriver::ExternalHarness
                })
    }

    /// Hands a cancellation of `conversation_id` to the external harness driving it. Returns
    /// `false` when no running harness owns the conversation (including a harness whose turn
    /// driver has already stopped), in which case the caller cancels it itself.
    pub(crate) fn cancel_via_external_harness(
        &self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        if !self.is_cancelled_via_external_harness(conversation_id, ctx) {
            return false;
        }
        let Some(sink) = &self.external_harness_cancel_sink else {
            return false;
        };
        log::info!(
            "event=cancel_routed_to_external_harness task_id={:?} terminal_id={:?} conversation_id={conversation_id}",
            self.ambient_agent_task_id,
            self.terminal_surface_id,
        );
        if sink.try_send(ExternalHarnessCancel).is_err() {
            log::info!(
                "event=cancel_fell_back_to_native task_id={:?} terminal_id={:?} conversation_id={conversation_id} reason=harness_stopped",
                self.ambient_agent_task_id,
                self.terminal_surface_id,
            );
            return false;
        }
        true
    }







}

#[cfg(test)]
#[path = "external_harness_tests.rs"]
mod tests;
