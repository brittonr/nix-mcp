use crate::common::cache_registry::CacheRegistry;
use crate::common::security::helpers::validation_error_to_mcp;
use crate::common::security::{validate_flake_ref, AuditLogger};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

use super::types::{FlakeMetadataArgs, FlakeShowArgs, PrefetchUrlArgs};

/// Tools for working with Nix flakes.
///
/// This struct provides operations for inspecting flake metadata, viewing flake
/// outputs, and prefetching remote content with integrity hashes. These tools
/// are essential for flake-based Nix workflows and reproducible builds.
///
/// # Available Operations
///
/// - **Flake Inspection**: [`flake_metadata`](Self::flake_metadata), [`flake_show`](Self::flake_show)
/// - **Content Fetching**: [`prefetch_url`](Self::prefetch_url)
///
/// # Caching Strategy
///
/// - URL prefetches: 24-hour TTL (hashes are content-addressed and stable)
/// - Flake metadata: No caching (metadata changes with updates)
/// - Flake outputs: No caching (outputs change with flake updates)
///
/// # Timeouts
///
/// - `flake_metadata`: 30 seconds (metadata fetch and parsing)
/// - `flake_show`: 30 seconds (output evaluation is fast)
/// - `prefetch_url`: 60 seconds (downloads may take time)
///
/// # Security
///
/// All inputs are validated:
/// - Flake references checked for shell metacharacters
/// - URLs validated for protocol, encoding, and length
/// - All operations include audit logging
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::nix::FlakeTools;
/// use onix_mcp::nix::types::FlakeMetadataArgs;
/// use rmcp::handler::server::wrapper::Parameters;
/// use std::sync::Arc;
///
/// # async fn example(tools: FlakeTools) -> Result<(), Box<dyn std::error::Error>> {
/// // Get metadata for a flake
/// let result = tools.flake_metadata(Parameters(FlakeMetadataArgs {
///     flake_ref: "github:nixos/nixpkgs".to_string(),
/// })).await?;
/// # Ok(())
/// # }
/// ```
pub struct FlakeTools {
    audit: Arc<AuditLogger>,
    caches: Arc<CacheRegistry>,
}

impl FlakeTools {
    /// Creates a new `FlakeTools` instance with audit logging and caching.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    /// * `caches` - Shared cache registry containing prefetch cache
    pub fn new(audit: Arc<AuditLogger>, caches: Arc<CacheRegistry>) -> Self {
        Self { audit, caches }
    }
}

#[tool_router]
impl FlakeTools {
    #[tool(
        description = "Get metadata about a flake (inputs, outputs, description)",
        annotations(read_only_hint = true)
    )]
    pub async fn flake_metadata(
        &self,
        Parameters(FlakeMetadataArgs { flake_ref }): Parameters<FlakeMetadataArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate flake reference
        validate_flake_ref(&flake_ref).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "flake_metadata",
            Some(serde_json::json!({"flake_ref": &flake_ref})),
            || async {
                with_timeout(&self.audit, "flake_metadata", 30, || async {
                    let output = tokio::process::Command::new("nix")
                        .args(["flake", "metadata", "--json", &flake_ref])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to get flake metadata: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to read flake: {}", stderr),
                            None,
                        ));
                    }

                    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse metadata: {}", e),
                                None,
                            )
                        })?;

                    let mut info = Vec::new();

                    if let Some(description) = metadata.get("description").and_then(|v| v.as_str())
                    {
                        info.push(format!("Description: {}", description));
                    }

                    if let Some(url) = metadata.get("url").and_then(|v| v.as_str()) {
                        info.push(format!("URL: {}", url));
                    }

                    if let Some(locked) = metadata.get("locked") {
                        if let Some(rev) = locked.get("rev").and_then(|v| v.as_str()) {
                            info.push(format!("Revision: {}", &rev[..12.min(rev.len())]));
                        }
                        if let Some(last_mod) = locked.get("lastModified").and_then(|v| v.as_u64())
                        {
                            info.push(format!("Last Modified: {}", last_mod));
                        }
                    }

                    if let Some(locks) = metadata.get("locks") {
                        if let Some(nodes) = locks.get("nodes").and_then(|v| v.as_object()) {
                            let inputs: Vec<String> = nodes
                                .keys()
                                .filter(|k| k.as_str() != "root")
                                .map(|k| k.to_string())
                                .collect();
                            if !inputs.is_empty() {
                                info.push(format!("\nInputs: {}", inputs.join(", ")));
                            }
                        }
                    }

                    Ok(CallToolResult::success(vec![Content::text(
                        info.join("\n"),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Show the outputs available in a flake (packages, apps, devShells, etc.)",
        annotations(read_only_hint = true)
    )]
    pub async fn flake_show(
        &self,
        Parameters(FlakeShowArgs { flake_ref }): Parameters<FlakeShowArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        let flake_ref = flake_ref.unwrap_or_else(|| ".".to_string());

        // Validate flake reference
        validate_flake_ref(&flake_ref).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "flake_show",
            Some(serde_json::json!({"flake_ref": &flake_ref})),
            || async {
                with_timeout(&self.audit, "flake_show", 30, || async {
                    let output = tokio::process::Command::new("nix")
                        .args(["flake", "show", &flake_ref, "--json"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix flake show: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to show flake: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Parse and format the flake structure
                    if let Ok(flake_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let mut result = format!("Flake Outputs for: {}\n\n", flake_ref);

                        fn format_outputs(
                            value: &serde_json::Value,
                            prefix: String,
                            result: &mut String,
                        ) {
                            if let serde_json::Value::Object(map) = value {
                                for (key, val) in map {
                                    if val.is_object()
                                        && val.as_object().unwrap().contains_key("type")
                                    {
                                        let type_str = val["type"].as_str().unwrap_or("unknown");
                                        result.push_str(&format!(
                                            "{}  {}: {}\n",
                                            prefix, key, type_str
                                        ));
                                    } else if val.is_object() {
                                        result.push_str(&format!("{}{}:\n", prefix, key));
                                        format_outputs(val, format!("{}  ", prefix), result);
                                    }
                                }
                            }
                        }

                        format_outputs(&flake_json, String::new(), &mut result);

                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    } else {
                        Ok(CallToolResult::success(vec![Content::text(
                            stdout.to_string(),
                        )]))
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Prefetch a URL and get its hash for use in Nix expressions")]
    pub async fn prefetch_url(
        &self,
        Parameters(PrefetchUrlArgs { url, hash_format }): Parameters<PrefetchUrlArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_url;

        // Validate URL
        validate_url(&url).map_err(validation_error_to_mcp)?;

        // Create cache key including format
        let cache_key = format!("{}:{}", url, hash_format.as_deref().unwrap_or("sri"));

        // Check cache first
        if let Some(cached_result) = self.caches.prefetch.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute with security features (audit logging + 60s timeout)
        let prefetch_cache = self.caches.prefetch.clone();
        let cache_key_clone = cache_key.clone();

        audit_tool_execution(&self.audit, "prefetch_url", Some(serde_json::json!({"url": &url})), || async move {
            with_timeout(&self.audit, "prefetch_url", 60, || async {
                let _format = hash_format.unwrap_or_else(|| "sri".to_string());

                let output = tokio::process::Command::new("nix")
                    .args(["store", "prefetch-file", &url])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to prefetch URL: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(format!("Prefetch failed: {}", stderr), None));
                }

                // Parse hash from stderr which contains: "Downloaded '...' to '...' (hash 'sha256-...')."
                let stderr = String::from_utf8_lossy(&output.stderr);
                let hash = if let Some(hash_start) = stderr.find("(hash '") {
                    let hash_part = &stderr[hash_start + 7..];
                    if let Some(hash_end) = hash_part.find("')") {
                        hash_part[..hash_end].to_string()
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                };

                let result = format!(
                    "URL: {}\nHash: {}\n\nUse in Nix:\nfetchurl {{\n  url = \"{}\";\n  hash = \"{}\";\n}}",
                    url, hash, url, hash
                );

                // Cache the result
                prefetch_cache.insert(cache_key_clone, result.clone());

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }).await
        }).await
    }
}
