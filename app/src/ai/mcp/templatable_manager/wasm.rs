use std::collections::{HashMap, HashSet};

use uuid::Uuid;
use warpui::{AppContext, ModelContext};

use super::TemplatableMCPServerManager;
use crate::ai::mcp::MCPServerUpdate;
use crate::ai::mcp::templatable::{CloudTemplatableMCPServer, TemplatableMCPServer};
use crate::ai::mcp::templatable_installation::{TemplatableMCPServerInstallation, VariableValue};
use crate::cloud_object::Space;
use crate::server::cloud_objects::update_manager::InitiatedBy;
use crate::server::ids::ServerId;

impl TemplatableMCPServerManager {
    /// Creates a new [`TemplatableMCPServerManager`] instance.
    pub fn new(
        _locally_installed_servers: HashMap<Uuid, TemplatableMCPServerInstallation>,
        _running_server_uuids: Vec<Uuid>,
        _running_legacy_servers: &[Uuid],
        _ctx: &mut ModelContext<Self>,
    ) -> Self {
        Default::default()
    }

    /// Gets a CloudTemplatableMCPServer by its UUID.
    /// Returns the CloudTemplatableMCPServer model if found, otherwise None.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn get_cloud_templatable_mcp_server(
        &self,
        _uuid: Uuid,
    ) -> Option<&CloudTemplatableMCPServer> {
        log::warn!("Getting a CloudTemplatableMCPServer by UUID is not supported in WASM");
        None
    }

    /// Gets a creator for a TemplatableMCPServer by its UUID.
    /// Returns the creator if found, otherwise None.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn get_creator(&self, _uuid: Uuid, _app: &AppContext) -> Option<String> {
        log::warn!("Getting a creator for a TemplatableMCPServer by UUID is not supported in WASM");
        None
    }

    /// Gets all TemplatableMCPServers currently in Warp Drive.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn get_all_templatable_mcp_servers(&self) -> Vec<&TemplatableMCPServer> {
        log::warn!("Getting all TemplatableMCPServers is not supported in WASM");
        vec![]
    }

    /// Gets a TemplatableMCPServer by its UUID.
    /// Returns the TemplatableMCPServer model if found, otherwise None.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn get_templatable_mcp_server(&self, _uuid: Uuid) -> Option<&TemplatableMCPServer> {
        log::warn!("Getting a TemplatableMCPServer by UUID is not supported in WASM");
        None
    }

    /// Creates a new TemplatableMCPServer in Warp Drive.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn create_templatable_mcp_server(
        &mut self,
        _server: TemplatableMCPServer,
        _space: Space,
        _initiated_by: InitiatedBy,
        _ctx: &mut ModelContext<Self>,
    ) {
        log::warn!("Creating a TemplatableMCPServer is not supported in WASM");
    }

    /// Spawns a new MCP server from a given [`TemplatableMCPServer`] instance.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn spawn_server(&mut self, _uuid: Uuid, _ctx: &mut ModelContext<Self>) {
        log::warn!("MCP server spawning not supported in WASM");
    }

    /// Shuts down a running MCP server.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn shutdown_server(&mut self, _uuid: Uuid, _ctx: &mut ModelContext<Self>) {
        log::warn!("MCP server shutdown not supported in WASM");
    }

    /// Deletes a locally installed MCP server installation.
    ///
    /// This is a no-op in WASM, as MCP servers are not supported in WASM.
    pub fn delete_templatable_mcp_server_installation(
        &mut self,
        _installation_uuid: Uuid,
        _ctx: &mut ModelContext<Self>,
    ) {
        log::warn!("Templatable MCP server installation deletion not supported in WASM");
    }

    pub fn install_from_template(
        &mut self,
        _templatable_mcp_server: TemplatableMCPServer,
        _variable_values: HashMap<String, VariableValue>,
        _start_automatically: bool,
        _ctx: &mut ModelContext<Self>,
    ) -> Option<TemplatableMCPServerInstallation> {
        log::warn!("Templatable MCP server installation not supported in WASM");
        None
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

    /// Delete oauth credentials from secure storage
    ///
    /// No-op in WASM, as MCP servers are not supported in WASM
    pub fn delete_credentials_from_secure_storage(
        &mut self,
        _sync_id: Uuid,
        _app: &mut warpui::AppContext,
    ) {
        log::warn!("Deleting credentials for MCP servers is not supported in WASM")
    }

    pub fn is_server_template_shared(&self, _template_uuid: Uuid, _app: &AppContext) -> bool {
        false
    }

    pub fn get_installation_by_template_uuid(
        &self,
        _template_uuid: Uuid,
    ) -> Option<&TemplatableMCPServerInstallation> {
        None
    }

    pub fn get_all_cloud_synced_mcp_servers(_ctx: &AppContext) -> HashMap<Uuid, String> {
        Default::default()
    }

    pub fn get_mcp_name(_uuid: &Uuid, _app: &AppContext) -> Option<String> {
        Default::default()
    }

    pub fn spawn_ephemeral_server(
        &mut self,
        _installation: TemplatableMCPServerInstallation,
        _ctx: &mut ModelContext<Self>,
    ) {
        log::warn!("Ephemeral MCP server spawning not supported in WASM");
    }

    pub fn purge_file_based_server_credentials(
        &mut self,
        _installation_hashes: &Vec<u64>,
        _ctx: &mut ModelContext<Self>,
    ) {
        log::warn!("Purging file-based server credentials not supported in WASM");
    }

}
