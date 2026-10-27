
use warpui::keymap::EditableBinding;
use warpui::AppContext;

use super::context_menu::ContextMenuAction;
// `renders_in_warp_notebook_viewer` is only consumed by non-wasm views
// (`code::view` resolves to `view.rs` off-wasm and to `wasm.rs` on-wasm, and the
// tooltips helper is `local_fs`-gated). Gate the re-export to match, otherwise it
// is flagged as an unused import on the wasm build where those consumers are absent.
#[cfg(not(target_family = "wasm"))]
pub use crate::util::openable_file_type::renders_in_warp_notebook_viewer;
pub use crate::util::openable_file_type::is_markdown_file;
use crate::cmd_or_ctrl_shift;

/// Display mode for markdown files shown via the header segmented control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownDisplayMode {
    Rendered,
    Raw,
}




#[derive(Debug, Clone)]
pub enum FileNotebookAction {
    FocusTerminalInput,
    ReloadFile,
    ContextMenu(ContextMenuAction),
    ToggleMarkdownDisplayMode(MarkdownDisplayMode),
}

impl From<ContextMenuAction> for FileNotebookAction {
    fn from(action: ContextMenuAction) -> Self {
        FileNotebookAction::ContextMenu(action)
    }
}





/// Keymap context set by a pane that can switch between the rendered and raw views of its file,
/// so the toggle shortcut is only available where the Rendered/Raw control is shown.
pub const MARKDOWN_DISPLAY_MODE_TOGGLE_CONTEXT: &str = "MarkdownDisplayModeToggle";

// The rendered and raw views are separate panes with their own actions, so each registers its own
// direction of the toggle. Sharing the name and description gives them a single, jointly
// rebindable entry in settings.
pub const TOGGLE_MARKDOWN_DISPLAY_MODE_BINDING_NAME: &str = "markdown:toggle_display_mode";
pub const TOGGLE_MARKDOWN_DISPLAY_MODE_BINDING_DESCRIPTION: &str =
    "Toggle Markdown rendered/raw view";
pub const TOGGLE_MARKDOWN_DISPLAY_MODE_KEY_BINDING: &str = "cmdorctrl-e";

pub fn init(app: &mut AppContext) {
    use warpui::keymap::macros::*;

    app.register_editable_bindings([
        EditableBinding::new(
            "notebookview:focus_terminal_input",
            "Focus Terminal Input from File",
            FileNotebookAction::FocusTerminalInput,
        )
        .with_context_predicate(id!("FileNotebookView"))
        .with_key_binding(cmd_or_ctrl_shift("l")),
        EditableBinding::new(
            "notebookview:reload_file",
            "Reload file",
            FileNotebookAction::ReloadFile,
        )
        .with_context_predicate(id!("FileNotebookView")),
        EditableBinding::new(
            TOGGLE_MARKDOWN_DISPLAY_MODE_BINDING_NAME,
            TOGGLE_MARKDOWN_DISPLAY_MODE_BINDING_DESCRIPTION,
            FileNotebookAction::ToggleMarkdownDisplayMode(MarkdownDisplayMode::Raw),
        )
        .with_context_predicate(id!("FileNotebookView") & id!(MARKDOWN_DISPLAY_MODE_TOGGLE_CONTEXT))
        .with_key_binding(TOGGLE_MARKDOWN_DISPLAY_MODE_KEY_BINDING),
    ])
}








#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
