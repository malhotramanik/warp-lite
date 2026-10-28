use std::collections::HashMap;

use uuid::Uuid;
use warpui::{AppContext, ModelContext};

use super::TemplatableMCPServerManager;
use crate::ai::mcp::templatable_installation::TemplatableMCPServerInstallation;

/// An MCP server integration that Warp ships with bundled skills for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpIntegration {
    Figma,
}

impl TemplatableMCPServerManager {
    /// MCP servers are never run in warp-lite.
    pub fn is_mcp_server_running(&self, _integration: McpIntegration) -> bool {
        false
    }

    /// Built-in MCP servers are never spawned in warp-lite.
    pub fn sync_builtin_servers(&mut self, _force_respawn: bool, _ctx: &mut ModelContext<Self>) {}

    /// MCP OAuth flows are not supported in warp-lite.
    pub fn handle_oauth_callback(&mut self, _url: &url::Url) -> anyhow::Result<()> {
        anyhow::bail!("MCP servers are not supported")
    }

    /// Creates a new [`TemplatableMCPServerManager`] instance.
    pub fn new(
        _locally_installed_servers: HashMap<Uuid, TemplatableMCPServerInstallation>,
        _running_server_uuids: Vec<Uuid>,
        _running_legacy_servers: &[Uuid],
        _ctx: &mut ModelContext<Self>,
    ) -> Self {
        Default::default()
    }










    /// Enables (starts) the installed Figma MCP server.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn enable_figma_mcp(&mut self, _ctx: &mut ModelContext<Self>) {
        log::warn!("Enabling Figma MCP server is not supported in WASM");
    }

    /// Installs the Figma MCP server from the MCP gallery.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn install_figma_from_gallery(&mut self, _ctx: &mut ModelContext<Self>) {
        log::warn!("Installing Figma from gallery is not supported in WASM");
    }





    pub fn get_mcp_name(_uuid: &Uuid, _app: &AppContext) -> Option<String> {
        Default::default()
    }



}
