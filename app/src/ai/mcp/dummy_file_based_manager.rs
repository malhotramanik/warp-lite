use std::path::{Path, PathBuf};

use uuid::Uuid;
use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

use super::MCPProvider;
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

    pub fn file_based_servers(&self) -> Vec<&TemplatableMCPServerInstallation> {
        vec![]
    }

}

impl Entity for FileBasedMCPManager {
    type Event = ();
}

impl SingletonEntity for FileBasedMCPManager {}
