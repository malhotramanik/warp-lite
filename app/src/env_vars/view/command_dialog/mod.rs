pub use cloud_object_models::EnvVarSecretCommand;
use warpui::ViewContext;

use super::env_var_collection::{EnvVarCollectionView, VariableRowIndex};
use crate::env_vars::EnvVarValue;

mod command_dialog_view;
pub(super) use command_dialog_view::EnvVarCommandDialog;

impl EnvVarCollectionView {
    pub(super) fn display_command_dialog(
        &mut self,
        index: Option<VariableRowIndex>,
        ctx: &mut ViewContext<Self>,
    ) {
        if let Some(VariableRowIndex(index)) = index
            && let EnvVarValue::Command(cmd) = &self.variable_rows[index].value
        {
            self.env_var_command_dialog
                .update(ctx, |dialog, ctx| dialog.load(cmd, ctx))
        }
        self.dialog_open_states.env_var_command_dialog_open = true;
        self.update_open_modal_state(ctx);
        ctx.focus(&self.env_var_command_dialog);
        ctx.notify();
    }


}
