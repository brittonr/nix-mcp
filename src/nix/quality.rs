use crate::common::security::audit::AuditLogger;
use crate::common::security::helpers::{
    audit_tool_execution, validation_error_to_mcp, with_timeout,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

use super::types::{FormatNixArgs, LintNixArgs, NixFmtArgs, ValidateNixArgs};

pub struct QualityTools {
    audit: Arc<AuditLogger>,
}

impl QualityTools {
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl QualityTools {
    #[tool(
        description = "Format Nix code using nixpkgs-fmt",
        annotations(idempotent_hint = true)
    )]
    pub async fn format_nix(
        &self,
        Parameters(FormatNixArgs { code }): Parameters<FormatNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::validate_nix_expression;

        // Validate Nix code
        validate_nix_expression(&code).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(&self.audit, "format_nix", Some(serde_json::json!({"code_length": code.len()})), || async {
            with_timeout(&self.audit, "format_nix", 30, || async {
                // Try nixpkgs-fmt first, fallback to alejandra
                let child = tokio::process::Command::new("nixpkgs-fmt")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn();

                let mut child = match child {
                    Ok(c) => c,
                    Err(_) => {
                        // Try alejandra as fallback
                        tokio::process::Command::new("alejandra")
                            .args(["--quiet", "-"])
                            .stdin(std::process::Stdio::piped())
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .spawn()
                            .map_err(|e| McpError::internal_error(
                                format!("Neither nixpkgs-fmt nor alejandra found. Install with: nix-shell -p nixpkgs-fmt\nError: {}", e),
                                None
                            ))?
                    }
                };

                // Write code to stdin
                if let Some(ref mut stdin) = child.stdin {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(code.as_bytes()).await
                        .map_err(|e| McpError::internal_error(format!("Failed to write to formatter: {}", e), None))?;
                }

                let output = child.wait_with_output().await
                    .map_err(|e| McpError::internal_error(format!("Formatter failed: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(format!("Formatting failed: {}", stderr), None));
                }

                let formatted = String::from_utf8_lossy(&output.stdout);
                Ok(CallToolResult::success(vec![Content::text(formatted.to_string())]))
            }).await
        }).await
    }

    #[tool(
        description = "Format Nix code using the project's formatter (typically nix fmt)",
        annotations(read_only_hint = false)
    )]
    pub async fn nix_fmt(
        &self,
        Parameters(NixFmtArgs { path }): Parameters<NixFmtArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::validate_path;

        // Validate path if provided
        if let Some(ref p) = path {
            validate_path(p).map_err(validation_error_to_mcp)?;
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_fmt",
            Some(serde_json::json!({"path": &path})),
            || async {
                with_timeout(&self.audit, "nix_fmt", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("fmt");

                    if let Some(p) = path {
                        cmd.arg(p);
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute nix fmt: {}", e), None)
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("nix fmt failed: {}", stderr),
                            None,
                        ));
                    }

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
                        result = "Code formatted successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Validate Nix code syntax and check for parse errors",
        annotations(idempotent_hint = true)
    )]
    pub async fn validate_nix(
        &self,
        Parameters(ValidateNixArgs { code }): Parameters<ValidateNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::validate_nix_expression;

        // Validate Nix code for dangerous patterns
        validate_nix_expression(&code).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "validate_nix",
            Some(serde_json::json!({"code_length": code.len()})),
            || async {
                with_timeout(&self.audit, "validate_nix", 30, || async {
                    // Use nix-instantiate --parse to validate syntax
                    let child = tokio::process::Command::new("nix-instantiate")
                        .args(["--parse", "-E"])
                        .arg(&code)
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to spawn nix-instantiate: {}", e),
                                None,
                            )
                        })?;

                    let output = child.wait_with_output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to validate: {}", e), None)
                    })?;

                    if output.status.success() {
                        Ok(CallToolResult::success(vec![Content::text(
                            "✓ Nix code is valid! No syntax errors found.".to_string(),
                        )]))
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Ok(CallToolResult::success(vec![Content::text(format!(
                            "✗ Syntax errors found:\n\n{}",
                            stderr
                        ))]))
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Lint Nix code with statix and/or deadnix to find issues and anti-patterns",
        annotations(idempotent_hint = true)
    )]
    pub async fn lint_nix(
        &self,
        Parameters(LintNixArgs { code, linter }): Parameters<LintNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::validate_nix_expression;

        // Validate Nix code for dangerous patterns
        validate_nix_expression(&code).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(&self.audit, "lint_nix", Some(serde_json::json!({"code_length": code.len(), "linter": &linter})), || async {
            with_timeout(&self.audit, "lint_nix", 30, || async {
                let linter = linter.unwrap_or_else(|| "both".to_string());
                let mut results = Vec::new();

                // Create a temporary file for the code
                let temp_dir = std::env::temp_dir();
                let temp_file = temp_dir.join(format!("nix_lint_{}.nix", std::process::id()));

                tokio::fs::write(&temp_file, &code).await
                    .map_err(|e| McpError::internal_error(format!("Failed to write temp file: {}", e), None))?;

        // Run statix if requested
        if linter == "statix" || linter == "both" {
            let output = tokio::process::Command::new("statix")
                .args(["check", temp_file.to_str().unwrap()])
                .output()
                .await;

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !stdout.is_empty() || !stderr.is_empty() {
                        results.push(format!("=== statix findings ===\n{}{}", stdout, stderr));
                    } else if output.status.success() {
                        results.push("=== statix findings ===\n✓ No issues found by statix".to_string());
                    }
                }
                Err(_) => {
                    results.push("=== statix findings ===\n(statix not installed - run: nix-shell -p statix)".to_string());
                }
            }
        }

        // Run deadnix if requested
        if linter == "deadnix" || linter == "both" {
            let output = tokio::process::Command::new("deadnix")
                .arg(temp_file.to_str().unwrap())
                .output()
                .await;

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !stdout.is_empty() || !stderr.is_empty() {
                        results.push(format!("=== deadnix findings ===\n{}{}", stdout, stderr));
                    } else if output.status.success() {
                        results.push("=== deadnix findings ===\n✓ No dead code found".to_string());
                    }
                }
                Err(_) => {
                    results.push("=== deadnix findings ===\n(deadnix not installed - run: nix-shell -p deadnix)".to_string());
                }
            }
        }

        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_file).await;

        let result_text = if results.is_empty() {
            "No linters were run. Use linter=\"statix\", \"deadnix\", or \"both\".".to_string()
        } else {
            results.join("\n\n")
        };

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
            }).await
        }).await
    }
}
