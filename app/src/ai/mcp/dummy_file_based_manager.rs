use std::path::Path;

use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

use crate::ai::mcp::templatable_installation::TemplatableMCPServerInstallation;

pub struct FileBasedMCPManager {}

impl FileBasedMCPManager {
    pub fn new(_ctx: &mut ModelContext<Self>) -> Self {
        Self {}
    }

    pub fn get_servers_for_working_directory(
        &self,
        _cwd: &Path,
        _app: &AppContext,
    ) -> Vec<&TemplatableMCPServerInstallation> {
        vec![]
    }


}

impl Entity for FileBasedMCPManager {
    type Event = ();
}

impl SingletonEntity for FileBasedMCPManager {}
