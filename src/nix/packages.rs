use crate::common::cache_registry::CacheRegistry;
use crate::common::caching::CachedExecutor;
use crate::common::security::audit::AuditLogger;
use crate::common::security::helpers::{
    audit_tool_execution, validation_error_to_mcp, with_timeout,
};
use crate::common::security::{validate_command, validate_flake_ref, validate_package_name};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

use super::types::{
    CommaArgs, ExplainPackageArgs, FindCommandArgs, GetPackageInfoArgs, NixLocateArgs,
    SearchPackagesArgs,
};

/// Tools for searching, locating, and querying Nix packages.
///
/// This struct provides cached access to expensive Nix operations like package searches
/// and file location lookups. All operations are thread-safe and use TTL-based caching
/// to balance freshness with performance.
///
/// # Caching Strategy
///
/// - `search_cache`: 10-minute TTL for package search results
/// - `package_info_cache`: 30-minute TTL for package metadata
/// - `locate_cache`: 5-minute TTL for file location queries
///
/// # Security
///
/// All inputs are validated before execution:
/// - Package names checked for injection attacks and path traversal
/// - Commands validated to prevent shell injection
/// - All operations logged via audit logger
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::nix::PackageTools;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let audit = Arc::new(/* audit logger */);
/// let caches = Arc::new(/* cache registry */);
/// let tools = PackageTools::new(audit, caches);
///
/// // Search for packages
/// // let result = tools.search_packages(Parameters(SearchPackagesArgs {
/// //     query: "ripgrep".to_string(),
/// //     limit: Some(10),
/// // })).await?;
/// # Ok(())
/// # }
/// ```
pub struct PackageTools {
    audit: Arc<AuditLogger>,
    caches: Arc<CacheRegistry>,
}

impl PackageTools {
    /// Creates a new `PackageTools` instance with audit logging and caching.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    /// * `caches` - Shared cache registry containing search, package_info, and locate caches
    pub fn new(audit: Arc<AuditLogger>, caches: Arc<CacheRegistry>) -> Self {
        Self { audit, caches }
    }
}

#[tool_router]
impl PackageTools {
    #[tool(
        description = "Search for packages in nixpkgs by name or description",
        annotations(read_only_hint = true)
    )]
    pub async fn search_packages(
        &self,
        Parameters(SearchPackagesArgs { query, limit }): Parameters<SearchPackagesArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate query input
        validate_package_name(&query).map_err(validation_error_to_mcp)?;

        // Use cached executor with formatted cache key
        let cached_executor = CachedExecutor::new(self.caches.search.clone());
        let audit = self.audit.clone();
        let query_clone = query.clone();
        let limit_value = limit.unwrap_or(10);

        cached_executor
            .execute_with_formatted_cache(
                vec![query.clone(), limit_value.to_string()],
                || async move {
                    let audit_inner = audit.clone();
                    // Execute with security features (audit logging + timeout)
                    audit_tool_execution(
                        &audit,
                        "search_packages",
                        Some(serde_json::json!({"query": &query_clone})),
                        || async move {
                            with_timeout(&audit_inner, "search_packages", 30, || async {
                                // Use nix search command
                                let output = tokio::process::Command::new("nix")
                                    .args(["search", "nixpkgs", &query_clone, "--json"])
                                    .output()
                                    .await
                                    .map_err(|e| {
                                        McpError::internal_error(
                                            format!("Failed to execute nix search: {}", e),
                                            None,
                                        )
                                    })?;

                                if !output.status.success() {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    return Err(McpError::internal_error(
                                        format!("nix search failed: {}", stderr),
                                        None,
                                    ));
                                }

                                let stdout = String::from_utf8_lossy(&output.stdout);
                                let results: serde_json::Value = serde_json::from_str(&stdout)
                                    .map_err(|e| {
                                        McpError::internal_error(
                                            format!("Failed to parse search results: {}", e),
                                            None,
                                        )
                                    })?;

                                // Format results nicely
                                let mut formatted_results = Vec::new();
                                if let Some(obj) = results.as_object() {
                                    for (i, (pkg_path, info)) in obj.iter().enumerate() {
                                        if i >= limit_value {
                                            break;
                                        }

                                        let description = info["description"]
                                            .as_str()
                                            .unwrap_or("No description");
                                        let version = info["version"].as_str().unwrap_or("unknown");

                                        formatted_results.push(format!(
                                            "Package: {}\nVersion: {}\nDescription: {}\n",
                                            pkg_path, version, description
                                        ));
                                    }
                                }

                                let result_text = if formatted_results.is_empty() {
                                    format!("No packages found matching '{}'", query_clone)
                                } else {
                                    format!(
                                        "Found {} packages matching '{}':\n\n{}",
                                        formatted_results.len(),
                                        query_clone,
                                        formatted_results.join("\n")
                                    )
                                };

                                Ok(result_text)
                            })
                            .await
                        },
                    )
                    .await
                },
            )
            .await
    }

    #[tool(
        description = "Get detailed information about a specific package",
        annotations(read_only_hint = true)
    )]
    pub async fn get_package_info(
        &self,
        Parameters(GetPackageInfoArgs { package }): Parameters<GetPackageInfoArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Check cache first
        if let Some(cached_result) = self.caches.package_info.get(&package) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute with security features (audit logging + timeout)
        let package_info_cache = self.caches.package_info.clone();
        let package_clone = package.clone();

        audit_tool_execution(
            &self.audit,
            "get_package_info",
            Some(serde_json::json!({"package": &package})),
            || async move {
                with_timeout(&self.audit, "get_package_info", 30, || async {
                    // Use nix eval to get package metadata
                    let output = tokio::process::Command::new("nix")
                        .args(["eval", &package, "--json"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix eval: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("nix eval failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                    // Cache the result
                    package_info_cache.insert(package_clone, stdout.clone());

                    Ok(CallToolResult::success(vec![Content::text(stdout)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get detailed information about a package (version, description, homepage, license, etc.)",
        annotations(read_only_hint = true)
    )]
    pub async fn explain_package(
        &self,
        Parameters(ExplainPackageArgs { package }): Parameters<ExplainPackageArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package name
        validate_package_name(&package).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "explain_package",
            Some(serde_json::json!({"package": &package})),
            || async {
                with_timeout(&self.audit, "explain_package", 30, || async {
                    // Normalize package reference
                    let pkg_ref = if package.contains('#') {
                        package.clone()
                    } else {
                        format!("nixpkgs#{}", package)
                    };

                    // Get package metadata using nix eval
                    let meta_attr = format!("{}.meta", pkg_ref);

                    let output = tokio::process::Command::new("nix")
                        .args(["eval", "--json", &meta_attr])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to get package info: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to evaluate package: {}", stderr),
                            None,
                        ));
                    }

                    let meta: serde_json::Value =
                        serde_json::from_slice(&output.stdout).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse metadata: {}", e),
                                None,
                            )
                        })?;

                    let mut info = Vec::new();
                    info.push(format!("Package: {}", package));

                    if let Some(version) = meta.get("version").and_then(|v| v.as_str()) {
                        info.push(format!("Version: {}", version));
                    }

                    if let Some(description) = meta.get("description").and_then(|v| v.as_str()) {
                        info.push(format!("Description: {}", description));
                    }

                    if let Some(homepage) = meta.get("homepage").and_then(|v| v.as_str()) {
                        info.push(format!("Homepage: {}", homepage));
                    }

                    if let Some(license) = meta.get("license") {
                        if let Some(name) = license.get("spdxId").and_then(|v| v.as_str()) {
                            info.push(format!("License: {}", name));
                        } else if let Some(name) = license.get("fullName").and_then(|v| v.as_str())
                        {
                            info.push(format!("License: {}", name));
                        }
                    }

                    if let Some(platforms) = meta.get("platforms").and_then(|v| v.as_array()) {
                        let platform_list: Vec<String> = platforms
                            .iter()
                            .filter_map(|p| p.as_str().map(String::from))
                            .take(5)
                            .collect();
                        if !platform_list.is_empty() {
                            info.push(format!(
                                "Platforms: {} (showing first 5)",
                                platform_list.join(", ")
                            ));
                        }
                    }

                    if let Some(maintainers) = meta.get("maintainers").and_then(|v| v.as_array()) {
                        let maint_list: Vec<String> = maintainers
                            .iter()
                            .filter_map(|m| {
                                m.get("name").and_then(|n| n.as_str()).map(String::from)
                            })
                            .take(3)
                            .collect();
                        if !maint_list.is_empty() {
                            info.push(format!("Maintainers: {}", maint_list.join(", ")));
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
        description = "Find which package provides a command using nix-locate",
        annotations(read_only_hint = true)
    )]
    pub async fn find_command(
        &self,
        Parameters(FindCommandArgs { command }): Parameters<FindCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate command name
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "find_command", Some(serde_json::json!({"command": &command})), || async {
            with_timeout(&self.audit, "find_command", 30, || async {
                // Try nix-locate first
                let output = tokio::process::Command::new("nix-locate")
                    .args(["--top-level", "--whole-name", &format!("/bin/{}", command)])
                    .output()
                    .await;

                match output {
                    Ok(output) if output.status.success() => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let packages: Vec<&str> = stdout.lines()
                            .filter_map(|line| line.split_whitespace().next())
                            .take(10)
                            .collect();

                        if packages.is_empty() {
                            Ok(CallToolResult::success(vec![Content::text(
                                format!("Command '{}' not found in any package.\n\nTry:\n- nix search nixpkgs {}", command, command)
                            )]))
                        } else {
                            let result = format!(
                                "Command '{}' is provided by:\n\n{}\n\nInstall with:\n  nix-shell -p {}",
                                command,
                                packages.iter().map(|p| format!("  - {}", p)).collect::<Vec<_>>().join("\n"),
                                packages[0]
                            );
                            Ok(CallToolResult::success(vec![Content::text(result)]))
                        }
                    }
                    _ => {
                        // Fallback: provide instructions
                        Ok(CallToolResult::success(vec![Content::text(
                            format!(
                                "nix-locate not available. Install with: nix-shell -p nix-index\n\n\
                                To find command '{}' manually:\n\
                                1. nix search nixpkgs {}\n\
                                2. Try common packages: nix-shell -p {}\n\
                                3. Use https://search.nixos.org/packages to search",
                                command, command, command
                            )
                        )]))
                    }
                }
            }).await
        }).await
    }

    #[tool(
        description = "Find which package provides a specific file path using nix-locate",
        annotations(read_only_hint = true)
    )]
    pub async fn nix_locate(
        &self,
        Parameters(NixLocateArgs { path, limit }): Parameters<NixLocateArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Basic validation
        if path.is_empty() || path.contains('\0') {
            return Err(McpError::invalid_params(
                "Invalid path".to_string(),
                Some(serde_json::json!({"path": path})),
            ));
        }

        // Create cache key including limit
        let cache_key = format!("{}:{}", path, limit.unwrap_or(20));

        // Check cache first
        if let Some(cached_result) = self.caches.locate.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Wrap tool logic with security
        let locate_cache = self.caches.locate.clone();
        let cache_key_clone = cache_key.clone();

        audit_tool_execution(
            &self.audit,
            "nix_locate",
            Some(serde_json::json!({"path": &path, "limit": &limit})),
            || async move {
                with_timeout(&self.audit, "nix_locate", 60, || async {
                    // Try local nix-locate first (needs pre-built database)
                    let output = tokio::process::Command::new("nix-locate")
                        .arg("--whole-name")
                        .arg(&path)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix-locate: {}. Install with: nix-shell -p nix-index\nThen build database: nix-index", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("command not found") || stderr.contains("No such file") {
                            return Ok(CallToolResult::success(vec![Content::text(
                                "nix-locate is not available. Install it with: nix-shell -p nix-index\n\
                                Then build the database with: nix-index\n\
                                This may take several minutes on first run.".to_string()
                            )]));
                        }
                        return Err(McpError::internal_error(
                            format!("nix-locate failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<&str> = stdout.lines().collect();

                    let result = if lines.is_empty() {
                        format!("No packages found providing '{}'", path)
                    } else {
                        let limit = limit.unwrap_or(20);
                        let results: Vec<&str> = lines.iter().take(limit).copied().collect();
                        let total = lines.len();

                        let mut output =
                            format!("Found {} package(s) providing '{}':\n\n", total, path);
                        output.push_str(&results.join("\n"));

                        if total > limit {
                            output.push_str(&format!(
                                "\n\n... and {} more results (showing top {})",
                                total - limit,
                                limit
                            ));
                        }
                        output
                    };

                    // Cache the result
                    locate_cache.insert(cache_key_clone, result.clone());

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Run a command without installing it using comma (automatically finds and runs commands from nixpkgs)"
    )]
    pub async fn comma(
        &self,
        Parameters(CommaArgs { command, args }): Parameters<CommaArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate command name
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "comma",
            Some(serde_json::json!({"command": &command, "args": &args})),
            || async {
                with_timeout(&self.audit, "comma", 300, || async {
                    // Use the actual comma command
                    let mut cmd = tokio::process::Command::new(",");
                    cmd.arg(&command);

                    if let Some(ref program_args) = args {
                        for arg in program_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await;

                    match output {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);

                            let mut result = String::new();
                            if !stdout.is_empty() {
                                result.push_str(&stdout);
                            }
                            if !stderr.is_empty() {
                                if !result.is_empty() {
                                    result.push('\n');
                                }
                                result.push_str(&stderr);
                            }

                            if result.is_empty() {
                                result = format!(
                                    "Command completed (exit code: {})",
                                    output.status.code().unwrap_or(0)
                                );
                            }

                            Ok(CallToolResult::success(vec![Content::text(result)]))
                        }
                        Err(_) => {
                            // Comma not available, provide installation instructions
                            Ok(CallToolResult::success(vec![Content::text(format!(
                                "The 'comma' tool is not available.\n\n\
                                Install with:\n\
                                - nix-env -iA nixpkgs.comma\n\
                                - Or add to your NixOS configuration: environment.systemPackages = [ pkgs.comma ];\n\n\
                                Comma requires nix-index. Install and update it:\n\
                                - nix-shell -p nix-index --run nix-index\n\n\
                                Alternatively, try:\n\
                                - nix run nixpkgs#{} -- {}",
                                command,
                                args.as_ref()
                                    .map(|a| a.join(" "))
                                    .unwrap_or_default()
                            ))]))
                        }
                    }
                })
                .await
            },
        )
        .await
    }
}
