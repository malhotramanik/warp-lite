use std::collections::HashMap;

#[cfg(feature = "local_fs")]
use serde::Deserialize;

#[cfg(feature = "local_fs")]
use crate::ai::mcp::JSONTransportType;


/// A single entry under `[mcp_servers.<name>]` in a Codex TOML file.
///
/// Codex servers are either STDIO (discriminated by a required `command` field)
/// or streamable HTTP (discriminated by a required `url` field).
/// See https://developers.openai.com/codex/mcp/ for more details on Codex's MCP configuration spec.
#[cfg(feature = "local_fs")]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CodexServerEntry {
    /// A local process server launched over stdin/stdout.
    Stdio {
        /// The command that starts the server. Acts as the discriminant.
        command: String,
        /// Arguments to pass to the server command.
        #[serde(default)]
        args: Vec<String>,
        /// Literal key=value environment variables set on the server process.
        #[serde(default)]
        env: HashMap<String, String>,
        /// Names of environment variables to forward from the calling shell.
        /// Each name `N` is lowered to a `"${N}"` placeholder in the merged `env` map.
        /// Explicit `env` values win over `env_vars` placeholders on collision.
        #[serde(default)]
        env_vars: Vec<String>,
        /// Working directory for the server process.
        /// Mapped to `working_directory` in Warp JSON.
        cwd: Option<String>,
    },
    /// A remote server reached over streamable HTTP.
    Http {
        /// The server URL. Acts as the discriminant.
        url: String,
        /// Name of an environment variable holding a bearer token.
        /// Lowered to `Authorization: "Bearer ${VAR}"` in the headers map.
        bearer_token_env_var: Option<String>,
        /// Static header values sent verbatim (e.g. `X-Client = "codex"`).
        /// Wins over `env_http_headers` on collision.
        #[serde(default)]
        http_headers: HashMap<String, String>,
        /// Map of header names to environment variable names.
        /// Each entry becomes `header = "${var_name}"` in the headers map.
        #[serde(default)]
        env_http_headers: HashMap<String, String>,
    },
}

#[cfg(feature = "local_fs")]
impl From<CodexServerEntry> for JSONTransportType {
    fn from(entry: CodexServerEntry) -> Self {
        match entry {
            CodexServerEntry::Stdio {
                command,
                args,
                env,
                env_vars,
                cwd,
            } => {
                // Build merged env: env_vars placeholders first.
                let mut merged_env: HashMap<String, String> = env_vars
                    .into_iter()
                    .map(|var_name| (var_name.clone(), format!("${{{var_name}}}")))
                    .collect();
                // Explicit env wins over env_vars placeholders on collision.
                for (k, v) in env {
                    merged_env.insert(k, v);
                }

                JSONTransportType::CLIServer {
                    command,
                    args,
                    env: merged_env,
                    working_directory: cwd,
                }
            }
            CodexServerEntry::Http {
                url,
                bearer_token_env_var,
                http_headers,
                env_http_headers,
            } => {
                // Build merged headers.
                // Merge order (later wins): bearer < env_http_headers < http_headers.
                let mut merged_headers: HashMap<String, String> = HashMap::new();

                if let Some(var_name) = bearer_token_env_var {
                    merged_headers.insert(
                        "Authorization".to_owned(),
                        format!("Bearer ${{{var_name}}}"),
                    );
                }
                for (header, var_name) in env_http_headers {
                    merged_headers.insert(header, format!("${{{var_name}}}"));
                }
                for (header, value) in http_headers {
                    merged_headers.insert(header, value);
                }

                JSONTransportType::SSEServer {
                    url,
                    headers: merged_headers,
                }
            }
        }
    }
}








#[cfg(test)]
#[path = "parsing_tests.rs"]
mod tests;
