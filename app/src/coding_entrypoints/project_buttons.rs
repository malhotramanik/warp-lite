
use warp_core::features::FeatureFlag;
use warpui::keymap::EditableBinding;
use warpui::AppContext;

use crate::util::bindings::{BindingGroup, CustomAction};


pub fn init(app: &mut AppContext) {
    use warpui::keymap::macros::*;

    app.register_editable_bindings([
        EditableBinding::new(
            "project_buttons:open_repository",
            "Open repository",
            ProjectButtonsAction::OpenRepository,
        )
        .with_context_predicate(id!("ProjectButons"))
        .with_group(BindingGroup::Folders.as_str())
        .with_custom_action(CustomAction::OpenRepository),
        EditableBinding::new(
            "project_buttons:create_new_project",
            "Create new project",
            ProjectButtonsAction::CreateProject,
        )
        .with_context_predicate(id!("ProjectButons"))
        .with_enabled(|| FeatureFlag::CreateProjectFlow.is_enabled())
        .with_mac_key_binding("cmd-shift-N")
        .with_linux_or_windows_key_binding("alt-shift-N"),
    ]);
}







#[derive(Clone, Copy, Debug)]
pub enum ProjectButtonsAction {
    OpenRepository,
    CreateProject,
}


