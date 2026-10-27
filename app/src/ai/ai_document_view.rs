
use warpui::keymap::{EditableBinding, FixedBinding};
use warpui::{
    AppContext, id,
};


pub fn init(app: &mut AppContext) {
    app.register_editable_bindings([EditableBinding::new(
        // Reuse the save file keybinding name and description
        // so that there's only one entry in settings reused for both cases.
        SAVE_FILE_BINDING_NAME,
        SAVE_FILE_BINDING_DESCRIPTION,
        AIDocumentAction::SendUpdatedPlan,
    )
    .with_context_predicate(id!("AIDocumentView") & !id!("IMEOpen"))
    .with_key_binding("cmdorctrl-s")]);

    // Allow closing the AI document pane with the same toggle keybinding used in Terminal
    app.register_fixed_bindings([FixedBinding::new(
        "cmdorctrl-alt-p",
        AIDocumentAction::Close,
        id!("AIDocumentView") & !id!("IMEOpen"),
    )]);
}


// Import keybinding constants from code view to ensure consistency
use crate::code::view::{SAVE_FILE_BINDING_DESCRIPTION, SAVE_FILE_BINDING_NAME};

#[derive(Debug, Clone, PartialEq)]
pub enum AIDocumentAction {
    Close,
    SendUpdatedPlan,
}









