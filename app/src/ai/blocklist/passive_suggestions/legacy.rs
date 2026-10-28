use std::path::PathBuf;
#[cfg(not(target_family = "wasm"))]
use std::process::Stdio;
use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use command::r#async::Command;
use parking_lot::FairMutex;
use serde_json::json;
use warp_core::features::FeatureFlag;
use warpui::r#async::SpawnedFutureHandle;
use warpui::{Entity, EntityId, ModelContext, ModelHandle, SingletonEntity};

#[cfg(not(target_family = "wasm"))]
use crate::ai::agent::PassiveSuggestionTrigger;
use crate::ai::agent::CancellationReason;
use crate::ai::blocklist::controller::response_stream::ResponseStreamId;
use crate::ai::blocklist::controller::{BlocklistAIController, BlocklistAIControllerEvent};
use crate::network::NetworkStatus;
use crate::settings::AISettings;
use crate::terminal::event::{BlockType, UserBlockCompleted};
use crate::terminal::model::session::active_session::ActiveSession;
use crate::terminal::model::terminal_model::TerminalModel;
use crate::terminal::model_events::{ModelEvent, ModelEventDispatcher};
use crate::workspaces::user_workspaces::UserWorkspaces;


#[derive(Clone, Debug)]
pub enum PassiveSuggestionsEvent {
}

pub struct PassiveSuggestionsModel {
    active_session: ModelHandle<ActiveSession>,
    terminal_model: Arc<FairMutex<TerminalModel>>,
    ai_controller: ModelHandle<BlocklistAIController>,
    terminal_view_id: EntityId,
    prompt_suggestions_future_handle: Option<SpawnedFutureHandle>,
    unit_test_generation_future_handle: Option<SpawnedFutureHandle>,
    code_diff_preflight_future_handle: Option<SpawnedFutureHandle>,
    code_diff_timeout_future_handle: Option<SpawnedFutureHandle>,
    pending_unit_test_stream_id: Option<ResponseStreamId>,
    pending_code_diff_stream_id: Option<ResponseStreamId>,
}

impl PassiveSuggestionsModel {
    pub fn new(
        active_session: ModelHandle<ActiveSession>,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        ai_controller: ModelHandle<BlocklistAIController>,
        model_event_dispatcher: &ModelHandle<ModelEventDispatcher>,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        ctx.subscribe_to_model(model_event_dispatcher, |me, _, event, ctx| {
            me.handle_model_event(event, ctx);
        });
        ctx.subscribe_to_model(&ai_controller, |me, _, event, _ctx| {
            me.handle_controller_event(event, _ctx);
        });

        Self {
            active_session,
            terminal_model,
            ai_controller,
            terminal_view_id,
            prompt_suggestions_future_handle: None,
            unit_test_generation_future_handle: None,
            code_diff_preflight_future_handle: None,
            code_diff_timeout_future_handle: None,
            pending_unit_test_stream_id: None,
            pending_code_diff_stream_id: None,
        }
    }

    pub fn is_passive_code_diff_being_generated(&self) -> bool {
        self.pending_code_diff_stream_id.is_some()
    }

    pub fn abort_pending_requests(
        &mut self,
        ctx: &mut ModelContext<Self>,
    ) -> Vec<ResponseStreamId> {
        let mut aborted_stream_ids = Vec::new();
        if let Some(handle) = self.prompt_suggestions_future_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.unit_test_generation_future_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.code_diff_preflight_future_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.code_diff_timeout_future_handle.take() {
            handle.abort();
        }
        if let Some(stream_id) = self.pending_unit_test_stream_id.take() {
            self.ai_controller.update(ctx, |controller, ctx| {
                controller.try_cancel_pending_response_stream(
                    &stream_id,
                    CancellationReason::ManuallyCancelled,
                    ctx,
                );
            });
            aborted_stream_ids.push(stream_id);
        }
        if let Some(stream_id) = self.pending_code_diff_stream_id.take() {
            self.ai_controller.update(ctx, |controller, ctx| {
                controller.try_cancel_pending_response_stream(
                    &stream_id,
                    CancellationReason::ManuallyCancelled,
                    ctx,
                );
            });
            aborted_stream_ids.push(stream_id);
        }
        aborted_stream_ids
    }

    fn handle_model_event(&mut self, event: &ModelEvent, ctx: &mut ModelContext<Self>) {
        match event {
            ModelEvent::AfterBlockStarted { .. } => {
                self.abort_pending_requests(ctx);
            }
            ModelEvent::AfterBlockCompleted(after_block_completed_event) => {
                if FeatureFlag::PromptSuggestionsViaMAA.is_enabled() {
                    self.abort_pending_requests(ctx);
                    return;
                }
                let BlockType::User(block_completed) = &after_block_completed_event.block_type
                else {
                    return;
                };
                self.handle_user_block_completed(block_completed, ctx);
            }
            _ => {}
        }
    }

    fn handle_controller_event(
        &mut self,
        event: &BlocklistAIControllerEvent,
        ctx: &mut ModelContext<Self>,
    ) {
        match event {
            BlocklistAIControllerEvent::FinishedReceivingOutput { stream_id, .. } => {
                if self
                    .pending_unit_test_stream_id
                    .as_ref()
                    .is_some_and(|pending| pending == stream_id)
                {
                    self.pending_unit_test_stream_id = None;
                }
                if self
                    .pending_code_diff_stream_id
                    .as_ref()
                    .is_some_and(|pending| pending == stream_id)
                {
                    self.pending_code_diff_stream_id = None;
                    if let Some(handle) = self.code_diff_timeout_future_handle.take() {
                        handle.abort();
                    }
                }
            }
            BlocklistAIControllerEvent::SentRequest { stream_id, .. } => {
                if self.pending_unit_test_stream_id.as_ref() == Some(stream_id)
                    || self.pending_code_diff_stream_id.as_ref() == Some(stream_id)
                    || self.pending_code_diff_stream_id.as_ref() == Some(stream_id)
                {
                    return;
                }
                // If a new request is sent that wasn't for passive suggestions, abort any passive suggestions requests.
                self.abort_pending_requests(ctx);
            }
            _ => {}
        }
    }

    fn handle_user_block_completed(
        &mut self,
        block_completed: &UserBlockCompleted,
        ctx: &mut ModelContext<Self>,
    ) {
        if block_completed.was_part_of_agent_interaction {
            return;
        }

        self.abort_pending_requests(ctx);

        // Startup commands run while bootstrapping an Oz cloud environment, so we skip
        // passive prompt suggestion generation for them to avoid unnecessary requests.
        let is_oz_environment_startup_command = FeatureFlag::CloudModeSetupV2.is_enabled()
            && self
                .terminal_model
                .lock()
                .block_list()
                .block_at(block_completed.index)
                .is_some_and(|block| block.is_oz_environment_startup_command());
        if is_oz_environment_startup_command {
            return;
        }

        if should_generate_unit_test_suggestion(block_completed, &self.terminal_model, ctx) {
            self.generate_unit_test_suggestion(block_completed.clone(), ctx);
        } else if should_generate_prompt_suggestions(block_completed, &self.terminal_model, ctx) {
        }
    }


    fn generate_unit_test_suggestion(
        &mut self,
        block_completed: UserBlockCompleted,
        ctx: &mut ModelContext<Self>,
    ) {
        #[cfg(target_family = "wasm")]
        {
            let (_, _) = (block_completed, ctx);
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let serialized_block = block_completed.serialized_block.get_with(|compute| {
                let model = self.terminal_model.lock();
                compute(model.block_list())
            });
            let Some(current_dir) = serialized_block.pwd.as_ref().map(PathBuf::from) else {
                return;
            };

            self.unit_test_generation_future_handle = Some(ctx.spawn(
                async move {
                    let output = Command::new("git")
                        .args(["show", "HEAD"])
                        .current_dir(current_dir)
                        .stdout(Stdio::piped())
                        .output()
                        .await;
                    if let Ok(output) = output {
                        return String::from_utf8_lossy(&output.stdout).to_string();
                    }
                    String::new()
                },
                |me, diff_output: String, ctx| {
                    me.unit_test_generation_future_handle = None;
                    if diff_output.is_empty() {
                        return;
                    }
                    let diff_json = json!({ "diffs": diff_output });
                    let request = me.ai_controller.update(ctx, |controller, ctx| {
                        controller.send_unit_test_suggestions_request(
                            diff_json.to_string(),
                            PassiveSuggestionTrigger::CommandRun,
                            ctx,
                        )
                    });
                    if let Ok((_, stream_id)) = request {
                        me.pending_unit_test_stream_id = Some(stream_id.clone());
                    }
                },
            ));
        }
    }


}

impl Entity for PassiveSuggestionsModel {
    type Event = PassiveSuggestionsEvent;
}

fn should_generate_prompt_suggestions(
    block_completed: &UserBlockCompleted,
    terminal_model: &FairMutex<TerminalModel>,
    ctx: &ModelContext<PassiveSuggestionsModel>,
) -> bool {
    let command_is_empty = block_completed
        .command
        .get_with(|compute| {
            let model = terminal_model.lock();
            compute(model.block_list())
        })
        .trim()
        .is_empty();
    if command_is_empty {
        return false;
    }
    if !NetworkStatus::as_ref(ctx).is_online() {
        return false;
    }

    AISettings::as_ref(ctx).is_prompt_suggestions_enabled(ctx)
        && UserWorkspaces::as_ref(ctx).is_prompt_suggestions_toggleable()
}

fn should_generate_unit_test_suggestion(
    block_completed: &UserBlockCompleted,
    terminal_model: &FairMutex<TerminalModel>,
    ctx: &ModelContext<PassiveSuggestionsModel>,
) -> bool {
    let enabled = AISettings::as_ref(ctx).is_code_suggestions_enabled(ctx)
        && UserWorkspaces::as_ref(ctx).is_code_suggestions_toggleable();

    let command = block_completed.command.get_with(|compute| {
        let model = terminal_model.lock();
        compute(model.block_list())
    });

    enabled
        && command.starts_with("git")
        && command.contains("commit")
        && block_completed
            .serialized_block
            .get_with(|compute| {
                let model = terminal_model.lock();
                compute(model.block_list())
            })
            .exit_code
            .was_successful()
}




