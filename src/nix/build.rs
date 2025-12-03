use crate::common::cache_registry::CacheRegistry;
use crate::common::security::audit::AuditLogger;
use crate::common::security::helpers::{
    audit_tool_execution, validation_error_to_mcp, with_timeout,
};
use crate::common::security::{validate_flake_ref, validate_package_name};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

use super::types::{
    DiffDerivationsArgs, GetBuildLogArgs, GetClosureSizeArgs, NixBuildArgs, NixosBuildArgs,
    ShowDerivationArgs, WhyDependsArgs,
};

/// Tools for building packages and analyzing dependencies.
///
/// This struct provides operations for building Nix packages, analyzing derivations,
/// understanding dependency relationships, and debugging build failures. All operations
/// include caching for expensive queries and comprehensive security validation.
///
/// # Available Operations
///
/// - **Building**: [`nix_build`](Self::nix_build), [`nixos_build`](Self::nixos_build)
/// - **Dependency Analysis**: [`why_depends`](Self::why_depends), [`get_closure_size`](Self::get_closure_size)
/// - **Derivation Inspection**: [`show_derivation`](Self::show_derivation), [`diff_derivations`](Self::diff_derivations)
/// - **Debugging**: [`get_build_log`](Self::get_build_log)
///
/// # Caching Strategy
///
/// - Closure sizes: 30-minute TTL (expensive computation)
/// - Derivations: 30-minute TTL (stable unless package changes)
///
/// # Timeouts
///
/// Build operations have extended timeouts:
/// - Regular builds: 300 seconds (5 minutes)
/// - NixOS builds: 600 seconds (10 minutes)
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::nix::{BuildTools, NixBuildArgs};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let audit = Arc::new(/* audit logger */);
/// let caches = Arc::new(/* cache registry */);
/// let tools = BuildTools::new(audit, caches);
///
/// // Dry-run build to see what would be built
/// // let result = tools.nix_build(Parameters(NixBuildArgs {
/// //     package: "nixpkgs#hello".to_string(),
/// //     dry_run: Some(true),
/// // })).await?;
/// # Ok(())
/// # }
/// ```
pub struct BuildTools {
    audit: Arc<AuditLogger>,
    caches: Arc<CacheRegistry>,
}

impl BuildTools {
    /// Creates a new `BuildTools` instance with audit logging and caching.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    /// * `caches` - Shared cache registry containing closure_size and derivation caches
    pub fn new(audit: Arc<AuditLogger>, caches: Arc<CacheRegistry>) -> Self {
        Self { audit, caches }
    }
}

#[tool_router]
impl BuildTools {
    #[tool(description = "Build a Nix package and show what will be built or the build output")]
    pub async fn nix_build(
        &self,
        Parameters(NixBuildArgs { package, dry_run }): Parameters<NixBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 300s timeout for builds)
        audit_tool_execution(
            &self.audit,
            "nix_build",
            Some(serde_json::json!({"package": &package, "dry_run": dry_run})),
            || async {
                with_timeout(&self.audit, "nix_build", 300, || async {
                    let dry_run = dry_run.unwrap_or(false);

                    let mut args = vec!["build"];
                    if dry_run {
                        args.push("--dry-run");
                    }
                    args.push(&package);
                    args.push("--json");

                    let output = tokio::process::Command::new("nix")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix build: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        let error_msg = if dry_run {
                            format!("Dry-run build check failed:\n\n{}", stderr)
                        } else {
                            format!("Build failed:\n\n{}", stderr)
                        };

                        return Ok(CallToolResult::success(vec![Content::text(error_msg)]));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);

                    if dry_run {
                        // For dry-run, parse what would be built
                        let result = if let Ok(json_output) =
                            serde_json::from_str::<serde_json::Value>(&stdout)
                        {
                            format!(
                                "Dry-run completed successfully.\n\nBuild plan:\n{}",
                                serde_json::to_string_pretty(&json_output)
                                    .unwrap_or_else(|_| stdout.to_string())
                            )
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            format!("Dry-run completed successfully.\n\n{}", stderr)
                        };
                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    } else {
                        // For actual build, show the result
                        if let Ok(json_output) = serde_json::from_str::<serde_json::Value>(&stdout)
                        {
                            let mut result = String::from("Build completed successfully!\n\n");

                            if let Some(arr) = json_output.as_array() {
                                for item in arr {
                                    if let Some(drv_path) =
                                        item.get("drvPath").and_then(|v| v.as_str())
                                    {
                                        result.push_str(&format!("Derivation: {}\n", drv_path));
                                    }
                                    if let Some(out_paths) =
                                        item.get("outputs").and_then(|v| v.as_object())
                                    {
                                        result.push_str("Outputs:\n");
                                        for (name, path) in out_paths {
                                            if let Some(path_str) = path.as_str() {
                                                result.push_str(&format!(
                                                    "  {}: {}\n",
                                                    name, path_str
                                                ));
                                            }
                                        }
                                    }
                                }
                            }

                            result.push_str("\nResult symlink created: ./result\n");
                            Ok(CallToolResult::success(vec![Content::text(result)]))
                        } else {
                            Ok(CallToolResult::success(vec![Content::text(format!(
                                "Build completed!\n\n{}",
                                stdout
                            ))]))
                        }
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Explain why one package depends on another (show dependency chain)",
        annotations(read_only_hint = true)
    )]
    pub async fn why_depends(
        &self,
        Parameters(WhyDependsArgs {
            package,
            dependency,
            show_all,
        }): Parameters<WhyDependsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package names
        validate_package_name(&package).map_err(validation_error_to_mcp)?;
        validate_package_name(&dependency).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "why_depends",
            Some(serde_json::json!({"package": &package, "dependency": &dependency})),
            || async {
                with_timeout(&self.audit, "why_depends", 60, || async {
                    let show_all = show_all.unwrap_or(false);

                    // First, build the package to get its store path
                    let build_output = tokio::process::Command::new("nix")
                        .args(["build", &package, "--json", "--no-link"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to build package: {}", e),
                                None,
                            )
                        })?;

                    if !build_output.status.success() {
                        let stderr = String::from_utf8_lossy(&build_output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to build package: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&build_output.stdout);
                    let build_json: serde_json::Value =
                        serde_json::from_str(&stdout).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse build output: {}", e),
                                None,
                            )
                        })?;

                    let package_path = build_json
                        .as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|item| item.get("outputs"))
                        .and_then(|outputs| outputs.get("out"))
                        .and_then(|out| out.as_str())
                        .ok_or_else(|| {
                            McpError::internal_error(
                                "Failed to get package output path".to_string(),
                                None,
                            )
                        })?;

                    // Build dependency to get its store path
                    let dep_build_output = tokio::process::Command::new("nix")
                        .args(["build", &dependency, "--json", "--no-link"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to build dependency: {}", e),
                                None,
                            )
                        })?;

                    if !dep_build_output.status.success() {
                        let stderr = String::from_utf8_lossy(&dep_build_output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to build dependency: {}", stderr),
                            None,
                        ));
                    }

                    let dep_stdout = String::from_utf8_lossy(&dep_build_output.stdout);
                    let dep_json: serde_json::Value =
                        serde_json::from_str(&dep_stdout).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse dependency build output: {}", e),
                                None,
                            )
                        })?;

                    let dependency_path = dep_json
                        .as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|item| item.get("outputs"))
                        .and_then(|outputs| outputs.get("out"))
                        .and_then(|out| out.as_str())
                        .ok_or_else(|| {
                            McpError::internal_error(
                                "Failed to get dependency output path".to_string(),
                                None,
                            )
                        })?;

                    // Now run nix why-depends
                    let mut args = vec!["why-depends", package_path, dependency_path];
                    if show_all {
                        args.push("--all");
                    }

                    let output = tokio::process::Command::new("nix")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix why-depends: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        // Check if it's because there's no dependency
                        if stderr.contains("does not depend on") {
                            return Ok(CallToolResult::success(vec![Content::text(format!(
                                "{} does not depend on {}",
                                package, dependency
                            ))]));
                        }

                        return Err(McpError::internal_error(
                            format!("why-depends failed: {}", stderr),
                            None,
                        ));
                    }

                    let result = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        result.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Show the derivation details of a package (build inputs, environment, etc.)",
        annotations(read_only_hint = true)
    )]
    pub async fn show_derivation(
        &self,
        Parameters(ShowDerivationArgs { package }): Parameters<ShowDerivationArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package/flake reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Create cache key (package is the only parameter)
        let cache_key = package.clone();

        // Check cache first
        if let Some(cached_result) = self.caches.derivation.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Clone cache and key for use in async closure
        let derivation_cache = self.caches.derivation.clone();
        let cache_key_clone = cache_key.clone();

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "show_derivation",
            Some(serde_json::json!({"package": &package})),
            || async move {
                with_timeout(&self.audit, "show_derivation", 30, || async {
                    let output = tokio::process::Command::new("nix")
                        .args(["derivation", "show", &package])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix derivation show: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to show derivation: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Try to parse and format nicely
                    if let Ok(drv_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let mut result = String::from("Derivation Details:\n\n");

                        // Get the first (and usually only) derivation
                        if let Some(obj) = drv_json.as_object() {
                            if let Some((drv_path, drv_info)) = obj.iter().next() {
                                result.push_str(&format!("Path: {}\n\n", drv_path));

                                if let Some(outputs) =
                                    drv_info.get("outputs").and_then(|v| v.as_object())
                                {
                                    result.push_str("Outputs:\n");
                                    for (name, info) in outputs {
                                        result.push_str(&format!("  - {}\n", name));
                                        if let Some(path) =
                                            info.get("path").and_then(|v| v.as_str())
                                        {
                                            result.push_str(&format!("    Path: {}\n", path));
                                        }
                                    }
                                    result.push('\n');
                                }

                                if let Some(inputs) =
                                    drv_info.get("inputDrvs").and_then(|v| v.as_object())
                                {
                                    result.push_str(&format!(
                                        "Build Dependencies: {} derivations\n",
                                        inputs.len()
                                    ));
                                }

                                if let Some(env) = drv_info.get("env").and_then(|v| v.as_object()) {
                                    result.push_str("\nKey Environment Variables:\n");
                                    for key in
                                        ["name", "version", "src", "builder", "system", "outputs"]
                                            .iter()
                                    {
                                        if let Some(value) = env.get(*key).and_then(|v| v.as_str())
                                        {
                                            result.push_str(&format!("  {}: {}\n", key, value));
                                        }
                                    }
                                }

                                result.push_str("\nFull JSON available for detailed inspection.");
                                // Only show first derivation in formatted view
                            }
                        }

                        // Cache the result
                        derivation_cache.insert(cache_key_clone.clone(), result.clone());

                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    } else {
                        let result = stdout.to_string();

                        // Cache the result
                        derivation_cache.insert(cache_key_clone, result.clone());

                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get the closure size of a package (total size including all dependencies)",
        annotations(read_only_hint = true)
    )]
    pub async fn get_closure_size(
        &self,
        Parameters(GetClosureSizeArgs {
            package,
            human_readable,
        }): Parameters<GetClosureSizeArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package/flake reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Create cache key including human_readable flag
        let cache_key = format!("{}:{}", package, human_readable.unwrap_or(true));

        // Check cache first
        if let Some(cached_result) = self.caches.closure_size.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Clone cache and key for use in async closure
        let closure_size_cache = self.caches.closure_size.clone();
        let cache_key_clone = cache_key.clone();

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "get_closure_size", Some(serde_json::json!({"package": &package})), || async move {
            with_timeout(&self.audit, "get_closure_size", 60, || async {
                let human_readable = human_readable.unwrap_or(true);

                // First build the package to get its store path
                let build_output = tokio::process::Command::new("nix")
                    .args(["build", &package, "--json", "--no-link"])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to build package: {}", e), None))?;

                if !build_output.status.success() {
                    let stderr = String::from_utf8_lossy(&build_output.stderr);
                    return Err(McpError::internal_error(format!("Failed to build package: {}", stderr), None));
                }

                let stdout = String::from_utf8_lossy(&build_output.stdout);
                let build_json: serde_json::Value = serde_json::from_str(&stdout)
                    .map_err(|e| McpError::internal_error(format!("Failed to parse build output: {}", e), None))?;

                let package_path = build_json
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("outputs"))
                    .and_then(|outputs| outputs.get("out"))
                    .and_then(|out| out.as_str())
                    .ok_or_else(|| McpError::internal_error("Failed to get package output path".to_string(), None))?;

                // Get closure size using nix path-info
                let mut args = vec!["path-info", "-S", package_path];
                if !human_readable {
                    args.push("--json");
                }

                let output = tokio::process::Command::new("nix")
                    .args(&args)
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to get path info: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(format!("Failed to get closure size: {}", stderr), None));
                }

                let result_text = if human_readable {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // Parse the output which is in format: /nix/store/... \t closure_size
                    if let Some(line) = stdout.lines().next() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let closure_size: u64 = parts[1].parse().unwrap_or(0);
                            let size_gb = closure_size as f64 / (1024.0 * 1024.0 * 1024.0);
                            let size_mb = closure_size as f64 / (1024.0 * 1024.0);

                            let human_size = if size_gb >= 1.0 {
                                format!("{:.2} GB", size_gb)
                            } else {
                                format!("{:.2} MB", size_mb)
                            };

                            format!(
                                "Package: {}\nClosure Size: {} ({} bytes)\n\nThis includes the package and all its dependencies.",
                                package, human_size, closure_size
                            )
                        } else {
                            stdout.to_string()
                        }
                    } else {
                        "No size information available".to_string()
                    }
                } else {
                    String::from_utf8_lossy(&output.stdout).to_string()
                };

                // Cache the result
                closure_size_cache.insert(cache_key_clone, result_text.clone());

                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            }).await
        }).await
    }

    #[tool(
        description = "Get the build log for a package (useful for debugging build failures)",
        annotations(read_only_hint = true)
    )]
    pub async fn get_build_log(
        &self,
        Parameters(GetBuildLogArgs { package }): Parameters<GetBuildLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package name
        validate_package_name(&package).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "get_build_log", Some(serde_json::json!({"package": &package})), || async {
            with_timeout(&self.audit, "get_build_log", 30, || async {
                // nix log can take either a package reference or a store path
                let output = tokio::process::Command::new("nix")
                    .args(["log", &package])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute nix log: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    // Check if it's because the package hasn't been built
                    if stderr.contains("does not have a known build log") || stderr.contains("no build logs available") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            format!("No build log available for '{}'.\n\nThis could mean:\n- The package hasn't been built yet (use nix_build first)\n- The build was done by a different user/system\n- The log has been garbage collected\n\nTry building the package first: nix_build(package=\"{}\")", package, package)
                        )]));
                    }

                    return Err(McpError::internal_error(format!("Failed to get build log: {}", stderr), None));
                }

                let log = String::from_utf8_lossy(&output.stdout);

                // Truncate very long logs
                let result = if log.len() > 50000 {
                    let truncated = &log[..50000];
                    format!("{}\n\n... [Log truncated - showing first 50KB of {} KB total]",
                        truncated, log.len() / 1024)
                } else {
                    log.to_string()
                };

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }).await
        }).await
    }

    #[tool(
        description = "Compare two derivations to understand what differs between packages (uses nix-diff)",
        annotations(read_only_hint = true)
    )]
    pub async fn diff_derivations(
        &self,
        Parameters(DiffDerivationsArgs {
            package_a,
            package_b,
        }): Parameters<DiffDerivationsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package names
        validate_package_name(&package_a).map_err(validation_error_to_mcp)?;
        validate_package_name(&package_b).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "diff_derivations", Some(serde_json::json!({"package_a": &package_a, "package_b": &package_b})), || async {
            with_timeout(&self.audit, "diff_derivations", 60, || async {
                // First, try to use nix-diff if available
                let nix_diff_check = tokio::process::Command::new("nix-diff")
                    .arg("--version")
                    .output()
                    .await;

                if nix_diff_check.is_err() {
                    // nix-diff not available, provide installation instructions
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("nix-diff is not installed.\n\nInstall with:\n  nix-shell -p nix-diff\n\nOr add to your flake devShell:\n  buildInputs = [ pkgs.nix-diff ];\n\nAlternatively, you can use show_derivation to inspect each package separately:\n- show_derivation(package=\"{}\")\n- show_derivation(package=\"{}\")", package_a, package_b)
                    )]));
                }

                // Build both packages to get their derivation paths
                let build_a = tokio::process::Command::new("nix")
                    .args(["build", &package_a, "--json", "--no-link", "--dry-run"])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to build package A: {}", e), None))?;

                if !build_a.status.success() {
                    let stderr = String::from_utf8_lossy(&build_a.stderr);
                    return Err(McpError::internal_error(format!("Failed to build package A: {}", stderr), None));
                }

                let build_b = tokio::process::Command::new("nix")
                    .args(["build", &package_b, "--json", "--no-link", "--dry-run"])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to build package B: {}", e), None))?;

                if !build_b.status.success() {
                    let stderr = String::from_utf8_lossy(&build_b.stderr);
                    return Err(McpError::internal_error(format!("Failed to build package B: {}", stderr), None));
                }

                // Parse derivation paths from JSON output
                let json_a: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&build_a.stdout))
                    .map_err(|e| McpError::internal_error(format!("Failed to parse build output A: {}", e), None))?;
                let json_b: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&build_b.stdout))
                    .map_err(|e| McpError::internal_error(format!("Failed to parse build output B: {}", e), None))?;

                let drv_a = json_a
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("drvPath"))
                    .and_then(|drv| drv.as_str())
                    .ok_or_else(|| McpError::internal_error("Failed to get derivation path A".to_string(), None))?;

                let drv_b = json_b
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("drvPath"))
                    .and_then(|drv| drv.as_str())
                    .ok_or_else(|| McpError::internal_error("Failed to get derivation path B".to_string(), None))?;

                // Run nix-diff
                let output = tokio::process::Command::new("nix-diff")
                    .args([drv_a, drv_b])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to run nix-diff: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let result = if !stdout.is_empty() {
                    format!("Differences between {} and {}:\n\n{}", package_a, package_b, stdout)
                } else if !stderr.is_empty() {
                    stderr.to_string()
                } else {
                    format!("Packages {} and {} have identical derivations (no differences found).", package_a, package_b)
                };

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }).await
        }).await
    }

    #[tool(description = "Build a NixOS machine configuration from a flake")]
    pub async fn nixos_build(
        &self,
        Parameters(NixosBuildArgs {
            machine,
            flake,
            use_nom,
        }): Parameters<NixosBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        audit_tool_execution(&self.audit, "nixos_build", Some(serde_json::json!({"machine": &machine, "flake": &flake_str})), || async {
            with_timeout(&self.audit, "nixos_build", 300, || async {
                let use_nom = use_nom.unwrap_or(false);
                let build_target = format!("{}#nixosConfigurations.{}.config.system.build.toplevel", flake_str, machine);

                let mut cmd = if use_nom {
                    // Check if nom is available
                    let nom_check = tokio::process::Command::new("which")
                        .arg("nom")
                        .output()
                        .await;

                    if nom_check.is_ok() && nom_check.unwrap().status.success() {
                        let mut c = tokio::process::Command::new("nom");
                        c.args(["build", &build_target]);
                        c
                    } else {
                        let mut c = tokio::process::Command::new("nix");
                        c.args(["build", &build_target]);
                        c
                    }
                } else {
                    let mut c = tokio::process::Command::new("nix");
                    c.args(["build", &build_target]);
                    c
                };

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute build command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Build failed for NixOS configuration '{}':\n\n{}{}", machine, stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Successfully built NixOS configuration '{}'.\n\n{}{}\n\nThe build result is in ./result/", machine, stdout, stderr)
                )]))
            }).await
        }).await
    }
}
