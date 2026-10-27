use std::sync::Arc;
use std::time::Duration;

use async_channel::Sender;
use futures_util::stream::AbortHandle;
use lazy_static::lazy_static;
use regex::Regex;
use settings::Setting as _;
use url::Url;
use warp_core::context_flag::ContextFlag;
use warp_editor::model::{CoreEditorModel, RichTextEditorModel};
use warp_errors::{report_error, report_if_error};
use warpui::accessibility::{AccessibilityContent, WarpA11yRole};
use warpui::r#async::{SpawnedFutureHandle, Timer};
use warpui::clipboard::ClipboardContent;
use warpui::elements::{
    Align, Clipped, ConstrainedBox, Container, CrossAxisAlignment, DispatchEventResult, Empty,
    EventHandler, Flex, MainAxisAlignment, MainAxisSize, MouseStateHandle, ParentElement,
    SavePosition, Shrinkable, Stack,
};
use warpui::keymap::{EditableBinding, FixedBinding};
use warpui::presenter::ChildView;
use warpui::ui_components::button::ButtonVariant;
use warpui::ui_components::components::{UiComponent, UiComponentStyles};
use warpui::{
    AppContext, BlurContext, Element, Entity, FocusContext, ModelAsRef, ModelHandle,
    SingletonEntity, TypedActionView, View, ViewContext, ViewHandle,
};

use self::details_bar::DetailsBar;
use super::active_notebook_data::{
    ActiveNotebook, ActiveNotebookData, Mode, TrashStatus,
};
use super::context_menu::{
    ContextMenuAction, ContextMenuState, show_rich_editor_context_menu,
    show_text_editor_context_menu,
};
use super::editor::view::RichTextEditorView;
use super::link::NotebookLinks;
use super::manager::NotebookManager;
use super::telemetry::NotebookTelemetryAction;
use super::{CloudNotebookModel, NotebookId, NotebookLocation, styles};
use crate::ai::blocklist::secret_redaction::find_secrets_in_text;
use crate::ai::document::ai_document_model::AIDocumentId;
use crate::appearance::Appearance;
use crate::cloud_object::grab_edit_access_modal::GrabEditAccessModal;
use crate::cloud_object::model::persistence::CloudModel;
use crate::cloud_object::model::view::{Editor, EditorState};
use crate::cloud_object::{CloudObject, CloudObjectEventEntrypoint, ObjectType, Space};
use crate::drive::drive_helpers::has_feature_gated_anonymous_user_reached_notebook_limit;
use crate::drive::export::ExportManager;
use crate::drive::items::WarpDriveItemId;
use crate::drive::sharing::ShareableObject;
use crate::drive::{CloudObjectTypeAndId, OpenWarpDriveObjectSettings};
use crate::editor::{
    EditorView, InteractionState,
};
use crate::features::FeatureFlag;
use crate::menu::{MenuItem, MenuItemFields};
use crate::network::NetworkStatus;
use crate::notebooks::CloudNotebook;
use crate::pane_group::focus_state::{PaneFocusHandle, PaneGroupFocusEvent};
use crate::pane_group::pane::view;
use crate::pane_group::{BackingView, PaneConfiguration, PaneEvent};
use crate::server::cloud_objects::update_manager::UpdateManager;
use crate::server::ids::{ClientId, SyncId};
use crate::server::telemetry::{
    CloudObjectTelemetryMetadata, NotebookActionEvent, NotebookTelemetryMetadata,
    SharingDialogSource, TelemetryCloudObjectType, TelemetryEvent,
};
use crate::settings::app_installation_detection::{
    UserAppInstallDetectionSettings, UserAppInstallStatus,
};
use crate::settings::{
    FontSettings, NotebookFontSize, decrease_notebook_font_size,
    increase_notebook_font_size,
};
use crate::terminal::safe_mode_settings::get_secret_obfuscation_mode;
use crate::ui_components::icons::{self, Icon};
#[cfg(target_family = "wasm")]
use crate::uri::web_intent_parser::open_url_on_desktop;
use crate::util::bindings::{self, CustomAction};
use crate::view_components::DismissibleToast;
use crate::workflows::{WorkflowSource, WorkflowType};
use crate::workspace::ToastStack;
use crate::workspaces::user_workspaces::UserWorkspaces;
use crate::{cmd_or_ctrl_shift, safe_info, send_telemetry_from_ctx};

mod details_bar;

#[cfg(test)]
#[path = "notebook_tests.rs"]
mod tests;

const EDIT_BUTTON_MARGIN: f32 = 6.;
const HEADER_MARGIN: f32 = 15.;
const BANNER_VERTICAL_MARGIN: f32 = 10.;

const CONFLICT_RESOLUTION_MESSAGE: &str = "This notebook could not be saved because changes were made while you were editing. Please copy your work and refresh.";
const REFRESH_BUTTON_TEXT: &str = "Refresh";

const FEATURE_NOT_AVAILABLE_MESSAGE: &str = "This notebook could not be saved to the server because the feature is temporarily unavailable. The changes are saved locally. Please retry later.";


/// The minimum size of an edit delta (in terms of the change in byte length of the serialized
/// Markdown) for it to be considered "meaningful". We're likely going to tune this over time:
/// * By refining the threshold
/// * By using a more advanced diff algorithm
const MEANINGFUL_EDIT_THRESHOLD: usize = 30;

#[cfg(not(test))]
const EDIT_WINDOW_DURATION: Duration = Duration::from_secs(60);
// Use a shorter window to make testing reasonable.
#[cfg(test)]
const EDIT_WINDOW_DURATION: Duration = Duration::from_millis(5);

lazy_static! {
    // This is used to replace any backslash followed by a punctuation character with just the punctuation character.
    static ref ESCAPE_PUNCTUATION_REGEX: Regex =
        Regex::new(r"\\([[:punct:]])").expect("Escape punctuation regex should be valid");
}

pub fn init(app: &mut AppContext) {
    use warpui::keymap::macros::*;

    app.register_editable_bindings([
        EditableBinding::new(
            "notebookview:increase_font_size",
            "Increase notebook font size",
            NotebookAction::IncreaseFontSize,
        )
        .with_context_predicate(id!("NotebookView") & id!("NotMatchNotebookToMonospaceSize"))
        .with_group(bindings::BindingGroup::Settings.as_str())
        .with_key_binding("cmdorctrl-="),
        EditableBinding::new(
            "notebookview:decrease_font_size",
            "Decrease notebook font size",
            NotebookAction::DecreaseFontSize,
        )
        .with_context_predicate(id!("NotebookView") & id!("NotMatchNotebookToMonospaceSize"))
        .with_group(bindings::BindingGroup::Settings.as_str())
        .with_key_binding("cmdorctrl--"),
        EditableBinding::new(
            "notebookview:reset_font_size",
            "Reset notebook font size",
            NotebookAction::ResetFontSize,
        )
        .with_context_predicate(id!("NotebookView") & id!("NotMatchNotebookToMonospaceSize"))
        .with_group(bindings::BindingGroup::Settings.as_str())
        .with_custom_action(CustomAction::ResetFontSize),
        EditableBinding::new(
            "notebookview:focus_terminal_input",
            "Focus Terminal Input from Notebook",
            NotebookAction::FocusTerminalInput,
        )
        .with_context_predicate(id!("NotebookView"))
        .with_key_binding(cmd_or_ctrl_shift("l")),
    ]);
    app.register_fixed_bindings([
        FixedBinding::new(
            "alt-enter",
            NotebookAction::ToggleMode,
            id!("NotebookView") & id!("NotebookIsEditable"),
        ),
        FixedBinding::custom(
            CustomAction::IncreaseFontSize,
            NotebookAction::IncreaseFontSize,
            "Increase font size",
            id!("NotebookView") & id!("NotMatchNotebookToMonospaceSize"),
        )
        .with_group(bindings::BindingGroup::Settings.as_str()),
        FixedBinding::custom(
            CustomAction::DecreaseFontSize,
            NotebookAction::DecreaseFontSize,
            "Decrease font size",
            id!("NotebookView") & id!("NotMatchNotebookToMonospaceSize"),
        )
        .with_group(bindings::BindingGroup::Settings.as_str()),
    ]);
}

struct NotebookUpdateRequestDebounceArg {}

#[derive(Default)]
struct ButtonMouseStates {
    conflict_resolution_refresh_button: MouseStateHandle,
    conflict_resolution_copy_all_button: MouseStateHandle,
    restore_from_trash_button: MouseStateHandle,
    copy_to_personal_drive_button: MouseStateHandle,
}

#[derive(Clone, Copy)]
enum NotebookSyncError {
    InConflict,
    FeatureNotAvailable,
}

/// A view that allows viewing/execution and editing of a Warp notebook.
/// We don't currently persist any data.
pub struct NotebookView {
    /// This is a stateful component that shows information about the notebook like its location
    /// breadcrumbs and the current editor. It's shown immediately above the title editor.
    details_bar: DetailsBar,
    title: ViewHandle<EditorView>,
    input: ViewHandle<RichTextEditorView>,
    grab_edit_access_modal: ViewHandle<GrabEditAccessModal>,
    focused: bool,
    last_focused_component: FocusedComponent,
    active_notebook_data: ModelHandle<ActiveNotebookData>,
    button_mouse_states: ButtonMouseStates,
    pane_configuration: ModelHandle<PaneConfiguration>,
    focus_handle: Option<PaneFocusHandle>,
    links: ModelHandle<NotebookLinks>,
    context_menu: ContextMenuState<Self>,

    /// Buffer length as of the last meaningful-edit check.
    last_content_length: usize,
    /// Whether or not the buffer has been edited since the last check.
    send_edit_telemetry: bool,
    edit_telemetry_handle: Option<AbortHandle>,

    /// Whether or not there are un-saved content edits.
    content_is_dirty: bool,
    /// Whether or not there are un-saved title edits.
    title_is_dirty: bool,
    /// Sender for requesting throttled saves.
    save_tx: Sender<NotebookUpdateRequestDebounceArg>,

    /// Save position for the bounds of this view.
    view_position_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotebookEvent {
    RunWorkflow {
        workflow: Arc<WorkflowType>,
        source: WorkflowSource,
    },
    EditWorkflow(SyncId),
    ViewInWarpDrive(WarpDriveItemId),
    Pane(PaneEvent),
    MoveToSpace {
        cloud_object_type_and_id: CloudObjectTypeAndId,
        new_space: Space,
    },
    OpenDriveObjectShareDialog {
        cloud_object_type_and_id: CloudObjectTypeAndId,
        invitee_email: Option<String>,
        source: SharingDialogSource,
    },
    AttachPlanAsContext(AIDocumentId),
}

impl From<PaneEvent> for NotebookEvent {
    fn from(event: PaneEvent) -> Self {
        NotebookEvent::Pane(event)
    }
}

#[derive(Debug, Clone)]
pub enum NotebookAction {
    Focus,
    ToggleMode,
    Close,
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,
    ConflictResolutionBannerRefreshClicked,
    FocusTerminalInput,
    ViewInWarpDrive(WarpDriveItemId),
    ContextMenu(ContextMenuAction), // right click context menu
    MoveToSpace {
        cloud_object_type_and_id: CloudObjectTypeAndId,
        new_space: Space,
    },
    Duplicate,
    Trash,
    Untrash,
    CopyToPersonal,
    CopyToClipboard,
    CopyLink(String),
    OpenLinkOnDesktop(Url),
    Export,
    AttachPlanAsContext(AIDocumentId),
}

impl From<ContextMenuAction> for NotebookAction {
    fn from(action: ContextMenuAction) -> Self {
        NotebookAction::ContextMenu(action)
    }
}

/// A focusable component of the notebook view. This is used to restore focus to the right
/// component when re-focusing the notebook view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedComponent {
    /// The title editor.
    Title,
    /// The body/input editor.
    Input,
}

impl NotebookView {

    /// Restore focus to the notebook view, by focusing its editor.
    pub fn focus(&mut self, ctx: &mut ViewContext<Self>) {
        // Emit accessibility content for the notebook, rather than the generic text input.
        if let Some(a11y_content) = self.accessibility_contents(ctx) {
            ctx.emit_a11y_content(a11y_content);
        }
        match self.last_focused_component {
            FocusedComponent::Title => self.focus_title(ctx),
            FocusedComponent::Input => self.focus_input(ctx),
        }
    }

    /// Focus the title view.
    fn focus_title(&mut self, ctx: &mut ViewContext<Self>) {
        log::trace!("Focusing notebook title editor");
        self.last_focused_component = FocusedComponent::Title;
        ctx.focus(&self.title);
        ctx.emit(NotebookEvent::Pane(PaneEvent::FocusSelf));
    }

    /// Focus the input editor.
    fn focus_input(&mut self, ctx: &mut ViewContext<Self>) {
        log::trace!("Focusing notebook body editor");
        self.last_focused_component = FocusedComponent::Input;
        ctx.focus(&self.input);
        ctx.emit(NotebookEvent::Pane(PaneEvent::FocusSelf));
    }

    /// Set the interaction states of the title and body editors.
    fn set_editor_interaction_state(
        &self,
        interaction_state: InteractionState,
        ctx: &mut ViewContext<Self>,
    ) {
        self.input.update(ctx, |input, ctx| {
            input.cursor_start(ctx);
            input.set_interaction_state(interaction_state, ctx);
        });
        self.title.update(ctx, |title, ctx| {
            title.set_interaction_state(interaction_state, ctx);
        });
    }

    fn title_from_editor(title_editor: &ViewHandle<EditorView>, app: &AppContext) -> String {
        let mut title = title_editor.as_ref(app).buffer_text(app);
        if title.is_empty() {
            title.push_str("Untitled");
        }
        title
    }

    /// The notebook title. This is pulled from the title editor, and may be more recent than
    /// what's been persisted to the server.
    fn title(&self, app: &AppContext) -> String {
        Self::title_from_editor(&self.title, app)
    }

    fn handle_focus_state_event(
        &mut self,
        event: &PaneGroupFocusEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        // For events that change the pane size, rebuild the editor layout to adjust soft-wrapping.
        if matches!(
            event,
            PaneGroupFocusEvent::FocusedPaneMaximizedChanged
                | PaneGroupFocusEvent::InSplitPaneChanged
        ) {
            self.input.update(ctx, |input, ctx| {
                input
                    .model()
                    .update(ctx, |model, ctx| model.rebuild_layout(ctx))
            });
        }
    }




    #[cfg(test)]
    pub fn context_menu(&mut self) -> &mut ContextMenuState<Self> {
        &mut self.context_menu
    }

    #[cfg(any(test, feature = "integration_tests"))]
    pub fn input_editor(&self) -> ViewHandle<RichTextEditorView> {
        self.input.clone()
    }

    #[cfg(test)]
    pub fn title_editor(&self) -> ViewHandle<EditorView> {
        self.title.clone()
    }










    /// The current Markdown content of this notebook.
    pub fn content(&self, ctx: &AppContext) -> String {
        self.input.as_ref(ctx).markdown(ctx)
    }

    /// Saves the notebook's current Markdown content, via the [`UpdateManager`].
    fn save_content(&mut self, ctx: &mut ViewContext<Self>) {
        self.send_edit_telemetry = true;
        let content = Arc::new(self.content(ctx));

        // Block saving if secrets are detected in the notebook when secret redaction is enabled.
        let secret_redaction = get_secret_obfuscation_mode(ctx);
        if secret_redaction.should_redact_secret() {
            let content_escaped = ESCAPE_PUNCTUATION_REGEX
                .replace_all(&content, "$1")
                .to_string();
            let content_secrets = find_secrets_in_text(&content_escaped);
            if !content_secrets.is_empty() {
                let window_id = ctx.window_id();
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    toast_stack.add_ephemeral_toast(
                        DismissibleToast::error(
                            "This notebook cannot be saved because its content contains secrets"
                                .to_string(),
                        ),
                        window_id,
                        ctx,
                    );
                });
                return;
            }
        }

        let active_notebook = self.active_notebook_data.as_ref(ctx).active_notebook();
        match active_notebook {
            // If the notebook has already been committed, then update the local
            // memory and server data via update manager
            ActiveNotebook::CommittedNotebook(id) => UpdateManager::handle(ctx)
                .update(ctx, move |update_manager, ctx| {
                    update_manager.update_notebook_data(content, id, ctx)
                }),
            // If the notebook hasn't been committed yet, create the notebook through update
            // manager, and update the active notebook
            ActiveNotebook::NewNotebook(notebook) => {
                if let Some(client_id) = notebook.id.into_client() {
                    UpdateManager::handle(ctx).update(ctx, |update_manager, ctx| {
                        update_manager.create_notebook(
                            client_id,
                            notebook.permissions.owner,
                            notebook.metadata.folder_id,
                            CloudNotebookModel {
                                title: notebook.model().title.clone(),
                                data: content.to_string(),
                                ai_document_id: notebook.model().ai_document_id,
                                conversation_id: notebook.model().conversation_id.clone(),
                            },
                            CloudObjectEventEntrypoint::Unknown,
                            true,
                            ctx,
                        );
                    });

                    self.active_notebook_data.update(ctx, |data, _| {
                        data.active_notebook =
                            ActiveNotebook::CommittedNotebook(SyncId::ClientId(client_id))
                    });
                }
            }
            ActiveNotebook::None => {
                report_error!("Tried to save notebook, but none were active")
            }
        }
    }

    /// Check for edit activity and send telemetry accordingly.
    ///
    /// This runs as a recursive async task that reports if an edit was made over the past
    /// [`EDIT_WINDOW_DURATION`]. The telemetry loop starts when entering edit mode, and stops
    /// when switching to view.
    fn check_edited(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(handle) = self.edit_telemetry_handle.take() {
            handle.abort();
        }

        // The notebook could have switched to view mode while the timer was pending, since Future
        // cancellation isn't guaranteed.
        if self.mode(ctx) != Mode::Editing {
            return;
        }

        if self.send_edit_telemetry {
            let content = self.content(ctx);
            let delta = content.len().abs_diff(self.last_content_length);
            self.last_content_length = content.len();
            self.send_edit_telemetry = false;

            send_telemetry_from_ctx!(
                TelemetryEvent::EditNotebook {
                    metadata: self.telemetry_metadata(ctx),
                    meaningful_change: delta > MEANINGFUL_EDIT_THRESHOLD
                },
                ctx
            );
        }

        // Schedule another check. If we stop editing in the meantime, either the mode check above
        // or the cancellation logic in `switch_to_view` will stop the timer loop.
        let next_check = ctx.spawn_abortable(
            Timer::after(EDIT_WINDOW_DURATION),
            |me, _, ctx| {
                me.check_edited(ctx);
            },
            |_, _| {},
        );
        self.edit_telemetry_handle = Some(next_check.abort_handle());
    }

    /// Checks if the user is the current known editor of the notebook, if they
    /// are, then sets the current editor to be None both locally and on the server
    fn try_give_up_edit_access(&self, ctx: &mut ViewContext<Self>) {
        let id = self.active_notebook_data.as_ref(ctx).id();
        UpdateManager::handle(ctx).update(ctx, |update_manager, ctx| {
            if let Some(id) = id {
                update_manager.give_up_notebook_edit_access(id, ctx);
            }
        });
        ctx.notify();
    }

    fn give_up_edit_access_and_start_viewing(&mut self, ctx: &mut ViewContext<Self>) {
        self.try_give_up_edit_access(ctx);
        self.switch_to_view(ctx);
    }

    /// Save any changes to the notebook.
    fn handle_save(&mut self, _: NotebookUpdateRequestDebounceArg, ctx: &mut ViewContext<Self>) {
        if self.content_is_dirty {
            self.save_content(ctx);
            self.content_is_dirty = false;
        }

        if self.title_is_dirty {
            self.update_title_in_server(ctx);
            self.title_is_dirty = false;
        }
    }




    fn switch_to_view(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(handle) = self.edit_telemetry_handle.take() {
            handle.abort();
        }
        self.active_notebook_data.update(ctx, |data, ctx| {
            data.mode = Mode::View;
            ctx.notify();
        });
        self.set_editor_interaction_state(InteractionState::Selectable, ctx);
        ctx.notify();
    }


    fn mode<C: ModelAsRef>(&self, ctx: &C) -> Mode {
        self.active_notebook_data.as_ref(ctx).mode
    }

    fn mode_app_ctx(&self, ctx: &AppContext) -> Mode {
        self.active_notebook_data.as_ref(ctx).mode
    }

    /// The ID of the notebook open in this view.
    pub fn notebook_id(&self, ctx: &impl ModelAsRef) -> Option<SyncId> {
        self.active_notebook_data.as_ref(ctx).id()
    }

    /// The server ID of this notebook, if it has been saved to the server.
    fn server_id(&self, ctx: &ViewContext<Self>) -> Option<NotebookId> {
        self.notebook_id(ctx)?.into_server().map(Into::into)
    }

    /// The current notebook metadata for telemetry.
    fn telemetry_metadata(&self, ctx: &ViewContext<Self>) -> NotebookTelemetryMetadata {
        let active_notebook_data = self.active_notebook_data.as_ref(ctx);
        let owner = active_notebook_data.owner(ctx);
        let space = active_notebook_data.space(ctx);
        NotebookTelemetryMetadata::new(
            self.server_id(ctx),
            owner.and_then(Into::into),
            owner.map_or(NotebookLocation::PersonalCloud, Into::into),
            space.map(Into::into),
        )
    }

    fn open_telemetry_metadata(&self, ctx: &ViewContext<Self>) -> NotebookTelemetryMetadata {
        self.telemetry_metadata(ctx).with_markdown_table_count(
            self.input
                .as_ref(ctx)
                .model()
                .as_ref(ctx)
                .markdown_table_count(ctx),
        )
    }

    #[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
    fn generic_telemetry_metadata(&self, ctx: &ViewContext<Self>) -> CloudObjectTelemetryMetadata {
        let notebook_data = self.active_notebook_data.as_ref(ctx);
        CloudObjectTelemetryMetadata {
            object_type: TelemetryCloudObjectType::Notebook,
            object_uid: notebook_data.id().and_then(SyncId::into_server),
            space: notebook_data.space(ctx).map(Into::into),
            team_uid: notebook_data.owner(ctx).and_then(Into::into),
        }
    }

    /// Send a [`NotebookTelemetryAction`] telemetry event.
    fn send_telemetry_action(&self, action: NotebookTelemetryAction, ctx: &mut ViewContext<Self>) {
        send_telemetry_from_ctx!(
            TelemetryEvent::NotebookAction(NotebookActionEvent {
                action,
                metadata: self.telemetry_metadata(ctx)
            }),
            ctx
        );
    }

    /// Puts the nodebook into edit mode and focuses the editor. The caller is responsible for
    /// checking that the notebook is editable.
    fn switch_to_edit(&mut self, ctx: &mut ViewContext<Self>) {
        self.active_notebook_data.update(ctx, |data, ctx| {
            data.mode = Mode::Editing;
            ctx.notify();
        });

        self.set_editor_interaction_state(InteractionState::Editable, ctx);

        // Reset edit-tracking state to prevent a false initial event.
        self.send_edit_telemetry = false;
        self.last_content_length = self.content(ctx).len();
        self.check_edited(ctx);
    }

    /// Sends a request to the server to grab notebook edit access, if the user is taking
    /// access from another user, we wait to actually switch them into edit mode. If we are
    /// not taking access, we go ahead and optimistically switch them in.
    fn grab_edit_access(&mut self, optimistically_grant_access: bool, ctx: &mut ViewContext<Self>) {
        let active_notebook = self.active_notebook_data.as_ref(ctx);
        if !active_notebook.trash_status(ctx).is_editable() {
            // Do not allow grabbing edit access if the notebook is trashed or feature flag is turned off.
            return;
        }
        if FeatureFlag::SharedWithMe.is_enabled() && !active_notebook.editability(ctx).can_edit() {
            return;
        }

        let id = active_notebook.id();
        UpdateManager::handle(ctx).update(ctx, |update_manager, ctx| {
            if let Some(id) = id {
                update_manager.grab_notebook_edit_access(id, optimistically_grant_access, ctx);
            }
        });

        // If we are optimistically granting access, go ahead and switch into edit mode.
        if optimistically_grant_access {
            self.switch_to_edit(ctx);
        }

        ctx.focus(&self.input);
        ctx.notify();
    }

    /// Called when a user hits the edit button from within a notebook view.
    /// If there's not another editor, grabs notebook edit access and directly switches it
    /// into edit mode. If there is another editor currently, displays the grab edit access
    /// dialog.
    pub fn grab_edit_access_or_display_access_dialog(&mut self, ctx: &mut ViewContext<Self>) {
        let active_notebook_data = self.active_notebook_data.as_ref(ctx);
        if active_notebook_data.has_conflicts(ctx) {
            // Do not attempt to grab edit access if there are conflicts.
            return;
        }

        let current_editor = active_notebook_data
            .current_editor(ctx)
            .unwrap_or(Editor::no_editor());
        if current_editor.state == EditorState::OtherUserActive {
            self.active_notebook_data.update(ctx, |data, ctx| {
                data.show_grab_edit_access_modal = true;
                ctx.notify();
            });
        } else {
            log::info!("Explicitly grabbing edit access, no active editor");
            self.grab_edit_access(true, ctx);
        }

        self.focus_input(ctx);
        ctx.notify();
    }

    /// Reset the notebook title editor's content as a system edit, which is not synced to the server.
    fn set_title(&mut self, notebook_title: &str, ctx: &mut ViewContext<Self>) {
        self.title.update(ctx, |title, ctx| {
            title.system_reset_buffer_text(notebook_title, ctx);
        });
    }

    fn set_content(&mut self, notebook: &CloudNotebook, ctx: &mut ViewContext<Self>) {
        // Initialize the content length so we can get a delta when editing.
        self.last_content_length = notebook.model().data.len();
        self.input.update(ctx, |input, ctx| {
            input.reset_with_markdown(notebook.model().data.as_str(), ctx);
        });

        self.switch_to_view(ctx);
    }

    fn increase_font_size(&mut self, ctx: &mut ViewContext<Self>) {
        report_if_error!(increase_notebook_font_size(ctx))
    }

    fn decrease_font_size(&mut self, ctx: &mut ViewContext<Self>) {
        report_if_error!(decrease_notebook_font_size(ctx))
    }

    fn apply_font_size_to_setting(&mut self, new_font_size: f32, ctx: &mut ViewContext<Self>) {
        FontSettings::handle(ctx).update(ctx, |font_settings, ctx| {
            report_if_error!(
                font_settings
                    .notebook_font_size
                    .set_value(new_font_size, ctx)
            )
        });
    }

    fn view_in_warp_drive(&mut self, id: WarpDriveItemId, ctx: &mut ViewContext<Self>) {
        ctx.emit(NotebookEvent::ViewInWarpDrive(id));
    }

    fn move_to_team_owner(
        &mut self,
        cloud_object_type_and_id: CloudObjectTypeAndId,
        new_space: Space,
        ctx: &mut ViewContext<Self>,
    ) {
        ctx.emit(NotebookEvent::MoveToSpace {
            cloud_object_type_and_id,
            new_space,
        });
    }

    fn duplicate_object(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(notebook_id) = self.notebook_id(ctx) {
            UpdateManager::handle(ctx).update(ctx, |update_manager, ctx| {
                update_manager.duplicate_object(
                    &CloudObjectTypeAndId::from_id_and_type(notebook_id, ObjectType::Notebook),
                    ctx,
                );
            });
            ctx.notify();
        }
    }

    fn trash_object(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(notebook_id) = self.notebook_id(ctx) {
            self.close(ctx);

            UpdateManager::handle(ctx).update(ctx, move |update_manager, ctx| {
                update_manager.trash_object(
                    CloudObjectTypeAndId::from_id_and_type(notebook_id, ObjectType::Notebook),
                    ctx,
                );
            });
        }
    }

    fn untrash_notebook(&self, ctx: &mut ViewContext<Self>) {
        if let Some(notebook_id) = self.notebook_id(ctx) {
            if has_feature_gated_anonymous_user_reached_notebook_limit(ctx) {
                return;
            }

            UpdateManager::handle(ctx).update(ctx, move |update_manager, ctx| {
                update_manager.untrash_object(
                    CloudObjectTypeAndId::from_id_and_type(notebook_id, ObjectType::Notebook),
                    ctx,
                );
            });
        }
    }

    /// Start exporting this notebook.
    fn export(&self, ctx: &mut ViewContext<Self>) {
        if let Some(notebook_id) = self.notebook_id(ctx) {
            let window_id = ctx.window_id();
            ExportManager::handle(ctx).update(ctx, |export_manager, ctx| {
                export_manager.export(
                    window_id,
                    &[CloudObjectTypeAndId::from_id_and_type(
                        notebook_id,
                        ObjectType::Notebook,
                    )],
                    ctx,
                )
            });
        }
    }

    /// Copy the current content into a new notebook in the user's personal space. This action is
    /// shown when the user is editing a notebook in a team space that gets trashed.
    ///
    /// It is _not_ the same as duplicating the notebook:
    /// * The current editor contents and state are preserved, and may be more recent than what's
    ///   been persisted
    /// * The duplicate naming scheme is not used
    /// * The new notebook is always in the user's personal drive
    fn copy_to_personal(&mut self, ctx: &mut ViewContext<Self>) {
        // In the case of an object being trashed by another user, ensure we use the most recent
        // local edits.
        let content = self.content(ctx);
        let title = self.title.as_ref(ctx).buffer_text(ctx);
        let active_notebook = self.active_notebook_data.as_ref(ctx).active_notebook();

        let ai_document_id = match active_notebook {
            ActiveNotebook::CommittedNotebook(id) => CloudModel::as_ref(ctx)
                .get_notebook(&id)
                .and_then(|n| n.model().ai_document_id),
            ActiveNotebook::NewNotebook(notebook) => notebook.model().ai_document_id,
            ActiveNotebook::None => None,
        };

        let copy_client_id = ClientId::new();
        let copy_sync_id = SyncId::ClientId(copy_client_id);

        let Some(personal_drive) = UserWorkspaces::as_ref(ctx).personal_drive(ctx) else {
            log::warn!("User drive not available for copying notebook");
            return;
        };

        UpdateManager::handle(ctx).update(ctx, |update_manager, ctx| {
            update_manager.create_notebook(
                copy_client_id,
                personal_drive,
                None,
                CloudNotebookModel {
                    title: title.clone(),
                    data: content,
                    ai_document_id,
                    conversation_id: None,
                },
                CloudObjectEventEntrypoint::Unknown,
                true,
                ctx,
            );
        });

        if let Some(previous_id) = self.notebook_id(ctx) {
            NotebookManager::handle(ctx).update(ctx, |notebook_manager, _| {
                notebook_manager.swap_notebook(previous_id, copy_sync_id);
            });
        }

        self.active_notebook_data
            .update(ctx, |active_notebook, ctx| {
                active_notebook.open_existing(copy_sync_id, ctx);
            });

        // Because the notebook was just created, and is in the user's personal space, grabbing
        // access must be safe.
        self.grab_edit_access(true, ctx);

        // Save the new notebook ID for session restoration.
        ctx.emit(NotebookEvent::Pane(PaneEvent::AppStateChanged));
    }

    fn copy_notebook_contents_to_clipboard(&mut self, ctx: &mut ViewContext<Self>) {
        self.input.update(ctx, |input, ctx| {
            input.model().update(ctx, |model, ctx| model.copy_all(ctx))
        });
    }

    fn online_only_operation_allowed(
        &self,
        cloud_object_type_and_id: CloudObjectTypeAndId,
        app: &AppContext,
    ) -> bool {
        if let Some(object) = CloudModel::as_ref(app).get_by_uid(&cloud_object_type_and_id.uid()) {
            return self.is_online(app)
                && cloud_object_type_and_id.has_server_id()
                && !object.metadata().has_pending_online_only_change();
        }

        false
    }

    pub fn notebook_link(&self, ctx: &AppContext) -> Option<String> {
        let id = self.notebook_id(ctx)?;

        if let Some(notebook) = CloudModel::as_ref(ctx).get_notebook(&id) {
            return notebook.object_link();
        }

        None
    }

    /// Items to show in the pane header overflow menu.
    fn overflow_menu_items(&self, ctx: &AppContext) -> Vec<MenuItem<NotebookAction>> {
        let active_notebook_data = self.active_notebook_data.as_ref(ctx);
        let access_level = active_notebook_data.access_level(ctx);
        let mut menu_items = Vec::new();

        if !active_notebook_data.is_on_server()
            || active_notebook_data.trash_status(ctx) != TrashStatus::Active
        {
            return menu_items;
        }

        // Add "Move to <team> space" to menu
        let team_spaces = UserWorkspaces::as_ref(ctx).team_spaces();

        if let (Some(space), Some(cloud_id)) =
            (active_notebook_data.space(ctx), active_notebook_data.id())
        {
            let cloud_object_type =
                CloudObjectTypeAndId::from_id_and_type(cloud_id, ObjectType::Notebook);
            let can_move = self.online_only_operation_allowed(cloud_object_type, ctx);

            if can_move {
                match space {
                    Space::Personal => {
                        menu_items.extend(team_spaces.iter().map(|space| {
                            MenuItemFields::new(format!("Move to {}", space.name(ctx)))
                                .with_on_select_action(NotebookAction::MoveToSpace {
                                    cloud_object_type_and_id: cloud_object_type,
                                    new_space: *space,
                                })
                                .with_icon(Icon::Move)
                                .into_item()
                        }));
                    }
                    Space::Shared => {} // TODO: Revisit these menu items with sharing in mind
                    Space::Team { .. } => {} // TODO: When we do team -> personal sharing
                }
            }
        }

        if let Some(ai_document_id) = self.active_notebook_data.as_ref(ctx).ai_document_id(ctx) {
            menu_items.push(
                MenuItemFields::new("Attach to active session")
                    .with_on_select_action(NotebookAction::AttachPlanAsContext(ai_document_id))
                    .with_icon(icons::Icon::Paperclip)
                    .into_item(),
            );
        }

        // Add "Copy Link" to menu
        if let Some(link) = self.notebook_link(ctx) {
            menu_items.push(
                MenuItemFields::new("Copy link")
                    .with_on_select_action(NotebookAction::CopyLink(link))
                    .with_icon(icons::Icon::Link)
                    .into_item(),
            );
        }

        if !warpui::platform::is_mobile_device()
            && !ContextFlag::HideOpenOnDesktopButton.is_enabled()
            && *UserAppInstallDetectionSettings::as_ref(ctx)
                .user_app_installation_detected
                .value()
                == UserAppInstallStatus::Detected
            && let Some(link) = self.notebook_link(ctx)
            && let Ok(url) = Url::parse(&link)
        {
            menu_items.push(
                MenuItemFields::new("Open on Desktop")
                    .with_on_select_action(NotebookAction::OpenLinkOnDesktop(url))
                    .with_icon(icons::Icon::Laptop)
                    .into_item(),
            );
        }

        // Add "Duplicate" to menu
        if active_notebook_data.space(ctx) != Some(Space::Shared) {
            menu_items.push(
                MenuItemFields::new("Duplicate")
                    .with_on_select_action(NotebookAction::Duplicate)
                    .with_icon(icons::Icon::Duplicate)
                    .into_item(),
            );
        }

        #[cfg(feature = "local_fs")]
        {
            menu_items.push(
                MenuItemFields::new("Export")
                    .with_on_select_action(NotebookAction::Export)
                    .with_icon(icons::Icon::Download)
                    .into_item(),
            );
        }

        // Add "Trash" to menu
        if self.is_online(ctx)
            && (!FeatureFlag::SharedWithMe.is_enabled() || access_level.can_trash())
        {
            menu_items.push(
                MenuItemFields::new("Trash")
                    .with_on_select_action(NotebookAction::Trash)
                    .with_icon(icons::Icon::Trash)
                    .into_item(),
            );
        }

        menu_items
    }


    fn is_online(&self, app: &AppContext) -> bool {
        NetworkStatus::as_ref(app).is_online()
    }



    /// Takes a `CloudNotebook` and loads it into the view.
    ///
    /// Namely, we reset the title and body's undo stack and we set the buffer to be
    /// that of the cloud notebook's content.
    ///
    /// The returned [`SpawnedFutureHandle`] guards asynchronous work to grab the baton and start
    /// editing if there is not already an editor.
    pub fn load(
        &mut self,
        notebook: CloudNotebook,
        settings: &OpenWarpDriveObjectSettings,
        ctx: &mut ViewContext<Self>,
    ) -> SpawnedFutureHandle {
        self.set_title(&notebook.model().title, ctx);
        self.set_content(&notebook, ctx);

        if let Some(server_id) = notebook.id.into_server() {
            self.pane_configuration
                .update(ctx, |pane_configuration, ctx| {
                    pane_configuration.set_shareable_object(
                        Some(ShareableObject::WarpDriveObject(server_id)),
                        ctx,
                    );
                });
        }

        self.active_notebook_data.update(ctx, |data, ctx| {
            data.open_existing(notebook.id, ctx);
        });
        self.input.update(ctx, |editor, ctx| {
            // TODO(ben): This is used for filtering in the embed UI, and should also probably be
            // owner-based.
            editor.set_space(notebook.space(ctx), ctx);
        });

        send_telemetry_from_ctx!(
            TelemetryEvent::OpenNotebook(self.open_telemetry_metadata(ctx)),
            ctx
        );

        // Once we've received metadata from the server, check if we can eagerly edit the notebook.
        let has_metadata = UpdateManager::as_ref(ctx).initial_load_complete();
        let baton_future = ctx.spawn(has_metadata, |me, _, ctx| {
            let active_notebook_data = me.active_notebook_data.as_ref(ctx);

            if FeatureFlag::SharedWithMe.is_enabled() && !active_notebook_data.editability(ctx).can_edit() {
                log::debug!("Notebook is view-only, opening in view mode");
            } else if active_notebook_data.has_conflicts(ctx) {
                log::debug!("Notebook has conflicts, opening in view mode");
            } else {
                let current_editor = active_notebook_data.current_editor(ctx);

                // If there's not currently an editor or the current editor has been idle, we want to automatically
                // switch the user into edit mode.
                match current_editor {
                    Some(editor) => {
                        let email = editor.email.unwrap_or_default();
                        match editor.state {
                            EditorState::None => {
                                log::info!("Optimistically grabbing edit access, no notebook editor");
                                me.grab_edit_access(true, ctx);
                            }
                            EditorState::CurrentUser => {
                                safe_info!(
                                    safe: ("Optimistically grabbing edit access, already the editor"),
                                    full: ("Optmisitically grabbing edit access, user {email} is already the editor")
                                );
                                me.grab_edit_access(true, ctx);
                            }
                            EditorState::OtherUserIdle => {
                                    safe_info!(
                                        safe: ("Optimistically grabbing edit access, editor is idle"),
                                        full: ("Optmisitically grabbing edit access, editor {email} is idle")
                                    );
                                    me.grab_edit_access(true, ctx);
                                }
                            EditorState::OtherUserActive => {
                                log::info!("Opening in view mode, notebook is being edited")
                            }
                        }
                    }
                    None => {
                        log::info!("Opening in view mode, unknown editor");
                    }
                }
            }
        });
        self.update_breadcrumbs(ctx);
        if let Some(invitee_email) = settings.invitee_email.clone() {
            let object_id_to_share = settings
                .focused_folder_id
                .map(|id| CloudObjectTypeAndId::Folder(SyncId::ServerId(id)))
                .unwrap_or(CloudObjectTypeAndId::Notebook(notebook.id));
            ctx.emit(NotebookEvent::OpenDriveObjectShareDialog {
                cloud_object_type_and_id: object_id_to_share,
                invitee_email: Some(invitee_email),
                source: SharingDialogSource::InviteeRequest,
            });
        } else if let Some(focused_folder_id) = settings.focused_folder_id.map(SyncId::ServerId) {
            self.view_in_warp_drive(
                WarpDriveItemId::Object(CloudObjectTypeAndId::Folder(focused_folder_id)),
                ctx,
            );
        }

        ctx.notify();
        baton_future
    }


    /// Updates the notebook title on the server with the current contents of the title editor.
    pub fn update_title_in_server(&mut self, ctx: &mut ViewContext<Self>) {
        let title: Arc<String> = self.title.as_ref(ctx).buffer_text(ctx).into();

        // Block saving if secrets are detected in the notebook title when secret redaction is enabled.
        let secret_redaction = get_secret_obfuscation_mode(ctx);
        if secret_redaction.should_redact_secret() {
            let title_escaped = ESCAPE_PUNCTUATION_REGEX
                .replace_all(&title, "$1")
                .to_string();
            let title_secrets = find_secrets_in_text(&title_escaped);
            if !title_secrets.is_empty() {
                let window_id = ctx.window_id();
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    toast_stack.add_ephemeral_toast(
                        DismissibleToast::error(
                            "This notebook cannot be saved because its title contains secrets"
                                .to_string(),
                        ),
                        window_id,
                        ctx,
                    );
                });
                return;
            }
        }

        let active_notebook = self.active_notebook_data.as_ref(ctx).active_notebook();
        match active_notebook {
            // If the notebook has already been committed, then update the local
            // memory and server data via update manager
            ActiveNotebook::CommittedNotebook(id) => UpdateManager::handle(ctx)
                .update(ctx, |update_manager, ctx| {
                    update_manager.update_notebook_title(title.clone(), id, ctx)
                }),
            // If the notebook hasn't been committed yet, create the notebook through update
            // manager, and update the active notebook
            ActiveNotebook::NewNotebook(notebook) => {
                if let Some(client_id) = notebook.id.into_client() {
                    UpdateManager::handle(ctx).update(ctx, |update_manager, ctx| {
                        update_manager.create_notebook(
                            client_id,
                            notebook.permissions.owner,
                            notebook.metadata.folder_id,
                            CloudNotebookModel {
                                title: title.to_string(),
                                data: notebook.model().data.to_owned(),
                                ai_document_id: notebook.model().ai_document_id,
                                conversation_id: notebook.model().conversation_id.clone(),
                            },
                            CloudObjectEventEntrypoint::Unknown,
                            true,
                            ctx,
                        );
                    });
                    self.active_notebook_data.update(ctx, |data, _| {
                        data.active_notebook =
                            ActiveNotebook::CommittedNotebook(SyncId::ClientId(client_id))
                    });
                }
            }
            ActiveNotebook::None => {
                report_error!("Tried to save notebook, but none were active")
            }
        }
    }

    /// Update the breadcrumbs for this notebook.
    fn update_breadcrumbs(&mut self, ctx: &mut ViewContext<Self>) {
        self.details_bar
            .update_breadcrumbs(self.active_notebook_data.as_ref(ctx), ctx);
        ctx.notify();
    }

    /// Save this notebook and give up edit access before detaching it from a pane.
    pub fn on_detach(&mut self, ctx: &mut ViewContext<Self>) {
        // If there are un-saved edits, persist them now, since the asynchronous update callback
        // is unlikely to run again.
        self.handle_save(NotebookUpdateRequestDebounceArg {}, ctx);

        // Give up notebook edit access on quitting.
        self.try_give_up_edit_access(ctx);
    }

    pub fn toggle_mode(&mut self, ctx: &mut ViewContext<Self>) {
        match self.mode(ctx) {
            Mode::Editing => {
                self.give_up_edit_access_and_start_viewing(ctx);
            }
            Mode::View => self.grab_edit_access_or_display_access_dialog(ctx),
        }
    }


    fn conflict_dialog_refresh_button_clicked(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(id) = self.notebook_id(ctx) else {
            return;
        };

        UpdateManager::handle(ctx).update(ctx, |update_manager, ctx| {
            update_manager.replace_object_with_conflict(&id.uid(), ctx);
        });

        // Load the server's version of the notebook now that the cloud model has been updated.
        // This will also switch back to edit mode if there isn't an active editor.
        if let Some(notebook) = CloudModel::as_ref(ctx).get_notebook(&id) {
            self.load(
                notebook.clone(),
                &OpenWarpDriveObjectSettings::default(),
                ctx,
            );
        }
        ctx.notify();
    }

    fn render_body(&self) -> Box<dyn Element> {
        let editor = self.input.clone();
        let saved_position = self.view_position_id.clone();

        EventHandler::new(styles::wrap_body(ChildView::new(&self.input).finish()))
            .on_right_mouse_down(move |ctx, _, position, _| {
                show_rich_editor_context_menu::<NotebookAction>(
                    ctx,
                    position,
                    &saved_position,
                    &editor,
                );
                DispatchEventResult::StopPropagation
            })
            .finish()
    }

    fn render_title(&self, app: &AppContext) -> Box<dyn Element> {
        let title_editor = self.title.clone();
        let saved_position = self.view_position_id.clone();
        let appearance = Appearance::as_ref(app);
        let title = EventHandler::new(Clipped::new(ChildView::new(&self.title).finish()).finish())
            .on_right_mouse_down(move |ctx, _, position, _| {
                show_text_editor_context_menu::<NotebookAction>(
                    ctx,
                    position,
                    &saved_position,
                    &title_editor,
                );
                DispatchEventResult::StopPropagation
            })
            .finish();

        let active_notebook_data = self.active_notebook_data.as_ref(app);

        let details = if active_notebook_data.trash_status(app).is_editable() {
            Some(
                self.details_bar
                    .render(active_notebook_data, appearance, app),
            )
        } else {
            None
        };

        styles::wrap_title(title, details)
    }

    fn render_trash_banner(&self, app: &AppContext) -> Option<Box<dyn Element>> {
        let deleted = match self.active_notebook_data.as_ref(app).trash_status(app) {
            TrashStatus::Active => return None,
            TrashStatus::Trashed => false,
            TrashStatus::Deleted => true,
        };
        let appearance = Appearance::as_ref(app);

        let mut stack = Stack::new();

        let text = if deleted {
            "You no longer have access to this notebook"
        } else {
            "Notebook was moved to trash"
        };
        stack.add_child(
            Align::new(
                Flex::row()
                    .with_children([
                        ConstrainedBox::new(
                            icons::Icon::Trash
                                .to_warpui_icon(appearance.theme().foreground())
                                .finish(),
                        )
                        .with_width(16.)
                        .with_height(16.)
                        .finish(),
                        appearance
                            .ui_builder()
                            .span(text)
                            .with_style(UiComponentStyles {
                                font_size: Some(appearance.ui_font_size() + 2.),
                                ..Default::default()
                            })
                            .build()
                            .with_padding_left(8.)
                            .finish(),
                    ])
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_main_axis_alignment(MainAxisAlignment::Center)
                    .with_main_axis_size(MainAxisSize::Max)
                    .finish(),
            )
            .finish(),
        );

        let action_row = if deleted {
            Shrinkable::new(1., Empty::new().finish()).finish()
        } else {
            let mut action_row = Flex::row()
                .with_main_axis_alignment(MainAxisAlignment::End)
                .with_main_axis_size(MainAxisSize::Max)
                .with_cross_axis_alignment(CrossAxisAlignment::Center);

            let active_notebook_data = self.active_notebook_data.as_ref(app);

            if !FeatureFlag::SharedWithMe.is_enabled()
                || active_notebook_data.access_level(app).can_trash()
            {
                let ui_builder = appearance.ui_builder().clone();
                action_row.add_child(
                    Align::new(
                        appearance
                            .ui_builder()
                            .button(
                                ButtonVariant::Basic,
                                self.button_mouse_states.restore_from_trash_button.clone(),
                            )
                            .with_tooltip(move || {
                                ui_builder
                                    .tool_tip("Restore notebook from trash".to_string())
                                    .build()
                                    .finish()
                            })
                            .with_text_label("Restore".to_string())
                            .build()
                            .on_click(|ctx, _, _| {
                                ctx.dispatch_typed_action(NotebookAction::Untrash)
                            })
                            .finish(),
                    )
                    .finish(),
                );
            }

            if active_notebook_data.space(app) != Some(Space::Personal) {
                let ui_builder = appearance.ui_builder().clone();
                action_row.add_child(
                    Container::new(
                        Align::new(
                            appearance
                                .ui_builder()
                                .button(
                                    ButtonVariant::Basic,
                                    self.button_mouse_states
                                        .copy_to_personal_drive_button
                                        .clone(),
                                )
                                .with_tooltip(move || {
                                    ui_builder
                                        .tool_tip(
                                            "Copy notebook contents into your personal workspace"
                                                .to_string(),
                                        )
                                        .build()
                                        .finish()
                                })
                                .with_text_label("Copy to Personal".to_string())
                                .build()
                                .on_click(|ctx, _, _| {
                                    ctx.dispatch_typed_action(NotebookAction::CopyToPersonal)
                                })
                                .finish(),
                        )
                        .finish(),
                    )
                    .with_padding_left(8.)
                    .finish(),
                );
            }
            action_row.finish()
        };

        stack.add_child(Align::new(action_row).right().finish());

        Some(
            Container::new(
                ConstrainedBox::new(stack.finish())
                    .with_min_height(40.)
                    .finish(),
            )
            .with_horizontal_padding(16.)
            .with_background(appearance.theme().surface_2())
            .finish(),
        )
    }

    fn render_sync_banner(
        &self,
        sync_error: NotebookSyncError,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let banner = Shrinkable::new(
            1.,
            appearance
                .ui_builder()
                .wrappable_text(
                    match sync_error {
                        NotebookSyncError::FeatureNotAvailable => FEATURE_NOT_AVAILABLE_MESSAGE,
                        NotebookSyncError::InConflict => CONFLICT_RESOLUTION_MESSAGE,
                    },
                    true,
                )
                .with_style(UiComponentStyles {
                    font_size: Some(appearance.ui_font_size() + 2.),
                    ..Default::default()
                })
                .build()
                .with_margin_bottom(BANNER_VERTICAL_MARGIN)
                .with_margin_top(BANNER_VERTICAL_MARGIN)
                .with_margin_right(HEADER_MARGIN)
                .with_margin_left(HEADER_MARGIN)
                .finish(),
        )
        .finish();

        let mut action_row = Flex::row()
            .with_main_axis_alignment(MainAxisAlignment::End)
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Center);

        let ui_builder = appearance.ui_builder().clone();
        action_row.add_child(
            Container::new(
                Align::new(
                    appearance
                        .ui_builder()
                        .button(
                            ButtonVariant::Basic,
                            self.button_mouse_states
                                .conflict_resolution_copy_all_button
                                .clone(),
                        )
                        .with_tooltip(move || {
                            ui_builder
                                .tool_tip("Copy notebook contents to your clipboard".to_string())
                                .build()
                                .finish()
                        })
                        .with_text_label("Copy All".to_string())
                        .build()
                        .on_click(|ctx, _, _| {
                            ctx.dispatch_typed_action(NotebookAction::CopyToClipboard)
                        })
                        .finish(),
                )
                .finish(),
            )
            .with_margin_bottom(BANNER_VERTICAL_MARGIN)
            .with_margin_right(HEADER_MARGIN)
            .with_margin_left(HEADER_MARGIN)
            .finish(),
        );

        if matches!(sync_error, NotebookSyncError::InConflict) {
            let ui_builder = appearance.ui_builder().clone();
            action_row.add_child(
                Container::new(
                    Align::new(
                        appearance
                            .ui_builder()
                            .button(
                                ButtonVariant::Basic,
                                self.button_mouse_states
                                    .conflict_resolution_refresh_button
                                    .clone(),
                            )
                            .with_tooltip(move || {
                                ui_builder
                                    .tool_tip("Refresh notebook".to_string())
                                    .build()
                                    .finish()
                            })
                            .with_text_label(REFRESH_BUTTON_TEXT.to_string())
                            .build()
                            .on_click(|ctx, _, _| {
                                ctx.dispatch_typed_action(
                                    NotebookAction::ConflictResolutionBannerRefreshClicked,
                                )
                            })
                            .finish(),
                    )
                    .finish(),
                )
                .with_margin_bottom(BANNER_VERTICAL_MARGIN)
                .with_margin_right(HEADER_MARGIN)
                .finish(),
            );
        }

        Container::new(
            Flex::column()
                .with_children([banner, action_row.finish()])
                .finish(),
        )
        .with_horizontal_padding(16.)
        .with_background(appearance.theme().surface_2())
        .finish()
    }
}

impl Entity for NotebookView {
    type Event = NotebookEvent;
}

impl View for NotebookView {
    fn ui_name() -> &'static str {
        "NotebookView"
    }

    fn accessibility_contents(&self, ctx: &AppContext) -> Option<AccessibilityContent> {
        Some(AccessibilityContent::new_without_help(
            format!("{} notebook", self.title(ctx)),
            WarpA11yRole::TextRole,
        ))
    }

    fn on_focus(&mut self, focus_ctx: &FocusContext, ctx: &mut ViewContext<Self>) {
        if focus_ctx.is_self_focused() {
            self.focused = true;
            ctx.notify();
        }
    }

    fn on_blur(&mut self, blur_ctx: &BlurContext, ctx: &mut ViewContext<Self>) {
        if blur_ctx.is_self_blurred() {
            self.focused = false;
            ctx.notify();
        }
    }

    fn render(&self, app: &AppContext) -> Box<dyn warpui::Element> {
        let mut content = Flex::column();
        content.extend(self.render_trash_banner(app));
        content.add_child(self.render_title(app));
        content.add_child(Shrinkable::new(1., self.render_body()).finish());

        let notebook = Align::new(content.finish()).top_left().finish();

        let mut stack = Stack::new();

        match self.mode_app_ctx(app) {
            // For editing mode, there is currently no use-case for focusing the notebook
            // view itself when clicking outside of the editor. We could change this behavior
            // if we need to in the future.
            Mode::Editing => stack.add_child(notebook),
            Mode::View => stack.add_child(
                EventHandler::new(notebook)
                    .on_left_mouse_down(|ctx, _, _| {
                        ctx.dispatch_typed_action(NotebookAction::Focus);
                        DispatchEventResult::StopPropagation
                    })
                    .finish(),
            ),
        };

        if self
            .active_notebook_data
            .as_ref(app)
            .show_grab_edit_access_modal
        {
            stack.add_child(ChildView::new(&self.grab_edit_access_modal).finish());
        }

        if self
            .active_notebook_data
            .as_ref(app)
            .feature_not_available()
        {
            stack.add_child(self.render_sync_banner(
                NotebookSyncError::FeatureNotAvailable,
                Appearance::as_ref(app),
            ));
        } else if self.active_notebook_data.as_ref(app).has_conflicts(app) {
            stack.add_child(
                self.render_sync_banner(NotebookSyncError::InConflict, Appearance::as_ref(app)),
            );
        }

        self.context_menu.render(&mut stack);

        SavePosition::new(stack.finish(), &self.view_position_id).finish()
    }

    fn keymap_context(&self, app: &AppContext) -> warpui::keymap::Context {
        let mut context = Self::default_keymap_context();

        match self.mode_app_ctx(app) {
            Mode::Editing => context.set.insert("NotebookEditing"),
            Mode::View => context.set.insert("NotebookViewing"),
        };

        if !FeatureFlag::SharedWithMe.is_enabled()
            || self
                .active_notebook_data
                .as_ref(app)
                .editability(app)
                .can_edit()
        {
            context.set.insert("NotebookIsEditable");
        }

        let font_settings = FontSettings::as_ref(app);
        if !font_settings.match_notebook_to_monospace_font_size.value() {
            context.set.insert("NotMatchNotebookToMonospaceSize");
        }

        context
    }
}


impl TypedActionView for NotebookView {
    type Action = NotebookAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            NotebookAction::Focus => ctx.focus_self(),
            NotebookAction::ToggleMode => self.toggle_mode(ctx),
            NotebookAction::Close => ctx.emit(NotebookEvent::Pane(PaneEvent::Close)),
            NotebookAction::ConflictResolutionBannerRefreshClicked => {
                self.conflict_dialog_refresh_button_clicked(ctx)
            }
            NotebookAction::IncreaseFontSize => self.increase_font_size(ctx),
            NotebookAction::DecreaseFontSize => self.decrease_font_size(ctx),
            NotebookAction::ResetFontSize => {
                self.apply_font_size_to_setting(NotebookFontSize::default_value(), ctx)
            }
            NotebookAction::ViewInWarpDrive(id) => self.view_in_warp_drive(*id, ctx),
            NotebookAction::FocusTerminalInput => {
                ctx.emit(NotebookEvent::Pane(PaneEvent::FocusActiveSession))
            }
            NotebookAction::ContextMenu(action) => {
                if matches!(action, ContextMenuAction::Open(_)) {
                    self.send_telemetry_action(NotebookTelemetryAction::OpenContextMenu, ctx);
                }
                self.context_menu.handle_action(action, ctx);
            }
            NotebookAction::Duplicate => self.duplicate_object(ctx),
            NotebookAction::Trash => self.trash_object(ctx),
            NotebookAction::Untrash => self.untrash_notebook(ctx),
            NotebookAction::CopyToPersonal => self.copy_to_personal(ctx),
            NotebookAction::CopyToClipboard => self.copy_notebook_contents_to_clipboard(ctx),
            NotebookAction::CopyLink(link) => {
                send_telemetry_from_ctx!(
                    TelemetryEvent::ObjectLinkCopied { link: link.clone() },
                    ctx
                );
                ctx.clipboard()
                    .write(ClipboardContent::plain_text(link.to_owned()));

                let window_id = ctx.window_id();
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    toast_stack.add_ephemeral_toast(
                        DismissibleToast::success("Link copied to clipboard".to_string()),
                        window_id,
                        ctx,
                    );
                });
            }
            NotebookAction::MoveToSpace {
                cloud_object_type_and_id,
                new_space,
            } => self.move_to_team_owner(*cloud_object_type_and_id, *new_space, ctx),
            #[cfg(target_family = "wasm")]
            NotebookAction::OpenLinkOnDesktop(url) => {
                send_telemetry_from_ctx!(
                    TelemetryEvent::WebCloudObjectOpenedOnDesktop {
                        object_metadata: self.generic_telemetry_metadata(ctx)
                    },
                    ctx
                );
                open_url_on_desktop(url);
            }
            #[cfg(not(target_family = "wasm"))]
            NotebookAction::OpenLinkOnDesktop(_) => {
                // No-op when not on wasm
            }
            NotebookAction::Export => self.export(ctx),
            NotebookAction::AttachPlanAsContext(id) => {
                ctx.emit(NotebookEvent::AttachPlanAsContext(*id))
            }
        };
    }
}

impl BackingView for NotebookView {
    type PaneHeaderOverflowMenuAction = NotebookAction;
    type CustomAction = ();
    type AssociatedData = ();

    fn pane_header_overflow_menu_items(&self, ctx: &AppContext) -> Vec<MenuItem<NotebookAction>> {
        self.overflow_menu_items(ctx)
    }

    fn handle_pane_header_overflow_menu_action(
        &mut self,
        action: &Self::PaneHeaderOverflowMenuAction,
        ctx: &mut ViewContext<Self>,
    ) {
        self.handle_action(action, ctx);
    }

    fn close(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.emit(NotebookEvent::Pane(PaneEvent::Close));
    }

    fn focus_contents(&mut self, ctx: &mut ViewContext<Self>) {
        self.focus(ctx);
    }

    fn render_header_content(
        &self,
        _ctx: &view::HeaderRenderContext<'_>,
        app: &AppContext,
    ) -> view::HeaderContent {
        view::HeaderContent::simple(self.pane_configuration.as_ref(app).title())
    }

    fn set_focus_handle(&mut self, focus_handle: PaneFocusHandle, ctx: &mut ViewContext<Self>) {
        ctx.subscribe_to_model(
            focus_handle.focus_state_handle(),
            |notebook, _handle, event, ctx| {
                notebook.handle_focus_state_event(event, ctx);
            },
        );

        self.focus_handle = Some(focus_handle.clone());
        self.context_menu.set_focus_handle(focus_handle);
    }
}
